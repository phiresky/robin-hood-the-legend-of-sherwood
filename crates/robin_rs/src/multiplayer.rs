//! Multiplayer transport — WebSocket-based server / client.
//!
//! The wire-format types ([`NetMsg`], [`NetEvent`], [`NetOutbound`],
//! [`NetChannels`]) and protocol constants live in
//! [`robin_engine::multiplayer`] so [`robin_engine::engine_manager::EngineManager`]
//! can own a `NetChannels` directly and route mutations through the
//! rollback-safe path.  This module owns only the platform-specific
//! transport submodules ([`native`] / [`wasm`]).

pub use robin_engine::multiplayer::{
    DEFAULT_PORT, FrameCursor, INPUT_DELAY_FRAMES, InitialSnapshot, NET_PROTOCOL_VERSION,
    NetChannels, NetEvent, NetMsg, NetOutbound, STATE_HASH_INTERVAL, decode_msg, encode_msg,
    new_frame_cursor,
};

pub mod lobby;

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(not(target_arch = "wasm32"))]
pub use native::{ClientHandle, ServerHandle, connect_client, start_server};

#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(target_arch = "wasm32")]
pub use wasm::{ClientHandle, ServerHandle, connect_client, start_server};

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::*;
    use crate::multiplayer::native::{connect_client, start_server};
    use robin_engine::player_command::{PlayerCommand, PlayerId, PlayerInput};
    use std::sync::mpsc::channel;
    use std::time::Duration;

    #[test]
    fn server_client_input_roundtrip() {
        // Start a server on an ephemeral port.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let addr = format!("127.0.0.1:{port}");

        // Server side.
        let (server_in_tx, server_in_rx) = channel::<NetEvent>();
        let (_server_out_tx, server_out_rx) = channel::<NetOutbound>();
        let server_cursor = new_frame_cursor();
        let server_snapshot = std::sync::Arc::new(std::sync::Mutex::new(None));
        let _server = start_server(
            &addr,
            "host".into(),
            42,
            server_in_tx,
            server_out_rx,
            server_cursor,
            server_snapshot,
            2,
        )
        .expect("start_server");

        // Brief sleep so the listener is accepting before we dial.
        std::thread::sleep(Duration::from_millis(50));

        // Client side.
        let (client_in_tx, client_in_rx) = channel::<NetEvent>();
        let (client_out_tx, client_out_rx) = channel::<NetOutbound>();
        let _client = connect_client(&addr, "alice".into(), client_in_tx, client_out_rx)
            .expect("connect_client");

        let assigned = loop {
            match client_in_rx.recv_timeout(Duration::from_secs(2)) {
                Ok(NetEvent::AssignedLocalSeat(p)) => break p,
                Ok(NetEvent::Note(_)) => continue,
                Ok(other) => panic!("unexpected pre-handshake event {other:?}"),
                Err(e) => panic!("timeout waiting for AssignedLocalSeat: {e}"),
            }
        };
        assert_eq!(assigned, PlayerId(1));

        let mut saw_join = false;
        for _ in 0..16 {
            match server_in_rx.recv_timeout(Duration::from_millis(200)) {
                Ok(NetEvent::Input { input, .. }) => {
                    if let PlayerCommand::ConnectSeat {
                        player_id,
                        ref nickname,
                        ..
                    } = input.command
                        && player_id == PlayerId(1)
                        && nickname == "alice"
                    {
                        saw_join = true;
                        break;
                    }
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
        assert!(
            saw_join,
            "server should have folded a ConnectSeat for the new client"
        );

        client_out_tx
            .send(NetOutbound::Input {
                origin_frame: 0,
                command: PlayerCommand::CrouchDown,
            })
            .unwrap();

        let (server_input, server_target) = loop {
            match server_in_rx.recv_timeout(Duration::from_secs(2)) {
                Ok(NetEvent::Input {
                    input,
                    target_frame,
                    ..
                }) if matches!(input.command, PlayerCommand::CrouchDown) => {
                    break (input, target_frame);
                }
                Ok(_) => continue,
                Err(e) => panic!("timeout waiting for server-side input echo: {e}"),
            }
        };
        assert_eq!(server_input.player_id, PlayerId(1));
        assert_eq!(server_target, INPUT_DELAY_FRAMES);

        let client_seen = loop {
            match client_in_rx.recv_timeout(Duration::from_secs(2)) {
                Ok(NetEvent::Input { input, .. })
                    if matches!(input.command, PlayerCommand::CrouchDown) =>
                {
                    break input;
                }
                Ok(_) => continue,
                Err(e) => panic!("timeout waiting for client-side input echo: {e}"),
            }
        };
        assert_eq!(client_seen.player_id, PlayerId(1));

        let _ = (PlayerInput::new(PlayerId(0), PlayerCommand::CrouchDown),);
    }
}
