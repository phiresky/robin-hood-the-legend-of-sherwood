//! WebAssembly (browser) WebSocket client for the multiplayer
//! transport.  Mirrors [`super::native`]'s [`connect_client`] surface
//! but uses the browser's `WebSocket` API instead of `tungstenite` —
//! `std::net` and synchronous reads aren't available in wasm.
//!
//! Server-side hosting is **not** supported on wasm: a browser tab
//! can't open a listening TCP socket.  Wasm clients can only connect
//! to a native `--server` running on a desktop / dedicated host.

use super::{NET_PROTOCOL_VERSION, NetEvent, NetMsg, NetOutbound, decode_msg, encode_msg};
use crate::player_command::PlayerId;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::js_sys;

/// Browser-side handle to an active client connection.  Owns the
/// JavaScript closures kept alive for the lifetime of the socket
/// (open / message / error / close); dropping the handle drops the
/// closures, which is fine because the socket itself keeps its
/// listeners until it closes.
pub struct ClientHandle {
    pub assigned_seat: Rc<RefCell<Option<PlayerId>>>,
    pub mission_seed: u64,
    /// Keeps the message-pump closure live.  The browser only invokes
    /// the closure while the WebSocket is open; we Drop it when the
    /// handle goes away.
    _on_message: Closure<dyn FnMut(web_sys::MessageEvent)>,
    _on_open: Closure<dyn FnMut(web_sys::Event)>,
    _on_close: Closure<dyn FnMut(web_sys::CloseEvent)>,
    _on_error: Closure<dyn FnMut(web_sys::Event)>,
    _socket: web_sys::WebSocket,
}

/// Connect to a multiplayer server from the browser.  Mirrors
/// [`super::native::connect_client`] but the I/O happens through the
/// browser event loop rather than a spawned thread.  Returns once the
/// `WebSocket` object is constructed — the handshake completes
/// asynchronously when the server answers, and the assigned seat is
/// reported through `incoming_tx` as `NetEvent::AssignedLocalSeat`
/// just like on native.
///
/// The returned `mission_seed` field is **0 until the Welcome
/// arrives**: unlike the native blocking handshake, we can't wait
/// here.  The transport emits both `AssignedLocalSeat` and
/// `MissionSeed` from the `Welcome` frame so the game loop can adopt
/// the authoritative host seed as soon as the browser handshake
/// completes.
pub fn connect_client(
    addr: &str,
    nickname: String,
    incoming_tx: Sender<NetEvent>,
    outgoing_rx: Receiver<NetOutbound>,
) -> Result<ClientHandle, std::io::Error> {
    let url = if addr.starts_with("ws://") || addr.starts_with("wss://") {
        addr.to_string()
    } else {
        format!("ws://{addr}/")
    };

    let socket = web_sys::WebSocket::new(&url)
        .map_err(|e| std::io::Error::other(format!("WebSocket::new: {e:?}")))?;
    socket.set_binary_type(web_sys::BinaryType::Arraybuffer);

    let assigned_seat = Rc::new(RefCell::new(None::<PlayerId>));

    // ── on_open: send the Hello handshake ──
    let on_open = {
        let socket = socket.clone();
        let nickname = nickname.clone();
        Closure::<dyn FnMut(_)>::new(move |_ev: web_sys::Event| {
            let hello = encode_msg(&NetMsg::Hello {
                protocol_version: NET_PROTOCOL_VERSION,
                nickname: nickname.clone(),
            });
            if let Err(e) = socket.send_with_u8_array(&hello) {
                tracing::error!("wasm-mp: send Hello failed: {e:?}");
            }
        })
    };
    socket.set_onopen(Some(on_open.as_ref().unchecked_ref()));

    // ── on_message: decode and route ──
    let on_message = {
        let incoming_tx = incoming_tx.clone();
        let assigned_seat = Rc::clone(&assigned_seat);
        Closure::<dyn FnMut(_)>::new(move |ev: web_sys::MessageEvent| {
            let data = ev.data();
            let bytes: Vec<u8> = if let Some(buf) = data.dyn_ref::<js_sys::ArrayBuffer>() {
                let array = js_sys::Uint8Array::new(buf);
                array.to_vec()
            } else {
                tracing::warn!("wasm-mp: non-binary frame received, ignoring");
                return;
            };
            match decode_msg(&bytes) {
                Ok(NetMsg::Welcome {
                    your_seat,
                    mission_seed,
                    host_nickname,
                }) => {
                    tracing::info!(
                        seat = your_seat.0,
                        seed = mission_seed,
                        host = %host_nickname,
                        "wasm-mp: welcomed by server"
                    );
                    *assigned_seat.borrow_mut() = Some(your_seat);
                    let _ = incoming_tx.send(NetEvent::AssignedLocalSeat(your_seat));
                    let _ = incoming_tx.send(NetEvent::MissionSeed(mission_seed));
                }
                Ok(NetMsg::InitialSnapshot {
                    frame,
                    engine_bytes,
                }) => {
                    let _ = incoming_tx.send(NetEvent::InitialSnapshot {
                        frame,
                        engine_bytes,
                    });
                }
                Ok(NetMsg::BeginSim {
                    frame,
                    start_epoch_ms,
                }) => {
                    let _ = incoming_tx.send(NetEvent::BeginSim {
                        frame,
                        start_epoch_ms,
                    });
                }
                Ok(NetMsg::BroadcastInput {
                    server_frame,
                    origin_frame,
                    target_frame,
                    input,
                }) => {
                    let _ = incoming_tx.send(NetEvent::Input {
                        server_frame,
                        origin_frame,
                        target_frame,
                        input,
                    });
                }
                Ok(NetMsg::Note(s)) => {
                    let _ = incoming_tx.send(NetEvent::Note(s));
                }
                Ok(NetMsg::StateHash {
                    frame,
                    hash,
                    clock_frame,
                    ms_until_next_frame,
                }) => {
                    let _ = incoming_tx.send(NetEvent::PeerStateHash {
                        frame,
                        hash,
                        clock_frame,
                        ms_until_next_frame,
                    });
                }
                Ok(NetMsg::ModalDismiss { kind, result }) => {
                    let _ = incoming_tx.send(NetEvent::ModalDismiss { kind, result });
                }
                Ok(other) => {
                    tracing::debug!(?other, "wasm-mp: ignoring unexpected wire message");
                }
                Err(e) => {
                    tracing::warn!("wasm-mp: decode error: {e}");
                }
            }
        })
    };
    socket.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

    // ── on_close + on_error: report disconnect ──
    let on_close = {
        let incoming_tx = incoming_tx.clone();
        Closure::<dyn FnMut(_)>::new(move |ev: web_sys::CloseEvent| {
            tracing::info!(
                code = ev.code(),
                reason = %ev.reason(),
                "wasm-mp: socket closed"
            );
            let _ = incoming_tx.send(NetEvent::Disconnected);
        })
    };
    socket.set_onclose(Some(on_close.as_ref().unchecked_ref()));

    let on_error = {
        let incoming_tx = incoming_tx.clone();
        Closure::<dyn FnMut(_)>::new(move |_ev: web_sys::Event| {
            tracing::warn!("wasm-mp: socket error event");
            let _ = incoming_tx.send(NetEvent::Note("wasm-mp: socket error".into()));
        })
    };
    socket.set_onerror(Some(on_error.as_ref().unchecked_ref()));

    // ── outgoing pump ──
    // Browsers don't support background threads (without
    // SharedArrayBuffer + workers, which we don't use here), so we
    // can't drain `outgoing_rx` in a parallel loop.  Instead, schedule
    // a recurring drain via `setInterval`.  Cheap enough at 25 Hz
    // (matches the sim tick rate); the channel is bounded by what
    // the game loop produced this frame so a single drain pulls
    // everything queued.
    schedule_outgoing_pump(socket.clone(), outgoing_rx);

    Ok(ClientHandle {
        assigned_seat,
        mission_seed: 0,
        _on_message: on_message,
        _on_open: on_open,
        _on_close: on_close,
        _on_error: on_error,
        _socket: socket,
    })
}

/// Schedule a 40 ms (~25 Hz) interval-driven drain of `outgoing_rx`.
/// Each tick pulls every queued [`NetOutbound`] and pushes the
/// encoded frames through the WebSocket.  The closure leaks
/// intentionally — `setInterval` keeps it alive until the page
/// closes.
fn schedule_outgoing_pump(socket: web_sys::WebSocket, outgoing_rx: Receiver<NetOutbound>) {
    use wasm_bindgen::closure::Closure;

    let outgoing_rx = Rc::new(RefCell::new(outgoing_rx));
    let socket = Rc::new(socket);
    let pump = Closure::<dyn FnMut()>::new({
        let outgoing_rx = Rc::clone(&outgoing_rx);
        let socket = Rc::clone(&socket);
        move || {
            let rx = outgoing_rx.borrow();
            while let Ok(outbound) = rx.try_recv() {
                let frame = match outbound {
                    NetOutbound::Input {
                        origin_frame,
                        command,
                    } => encode_msg(&NetMsg::Input {
                        origin_frame,
                        command,
                    }),
                    NetOutbound::StateHash { .. } => continue, // host-only
                    NetOutbound::InitialSnapshot { .. } => continue, // host-only
                    NetOutbound::ReadyToSim { frame } => encode_msg(&NetMsg::ReadyToSim { frame }),
                    NetOutbound::ModalDismiss { kind, result } => {
                        encode_msg(&NetMsg::ModalDismiss { kind, result })
                    }
                };
                if let Err(e) = socket.send_with_u8_array(&frame) {
                    tracing::warn!("wasm-mp: send failed: {e:?}");
                    break;
                }
            }
        }
    });

    if let Some(window) = web_sys::window() {
        let _ = window.set_interval_with_callback_and_timeout_and_arguments_0(
            pump.as_ref().unchecked_ref(),
            40,
        );
    }
    // Leak the closure so the browser can keep invoking it.
    pump.forget();
}

/// Server-side launch is not available in the browser.  The shape
/// matches the native [`super::native::ServerHandle`] so callers can
/// share one match arm; constructing it always fails here.
pub struct ServerHandle {
    pub local_seat: PlayerId,
    pub mission_seed: u64,
}

pub fn start_server(
    _addr: &str,
    _host_nickname: String,
    _mission_seed: u64,
    _incoming_tx: Sender<NetEvent>,
    _outgoing_rx: Receiver<NetOutbound>,
    _frame_cursor: super::FrameCursor,
    _initial_snapshot: super::InitialSnapshot,
    _expected_players: u32,
) -> Result<ServerHandle, std::io::Error> {
    Err(std::io::Error::other(
        "multiplayer server is not supported in the browser; \
         host on a native build and connect from wasm with --connect",
    ))
}
