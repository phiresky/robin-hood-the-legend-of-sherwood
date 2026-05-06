//! Keyboard shortcuts configuration sub-screen.
//!
//! A 640x480 window with the key config list box and five buttons.
//!
//! Buttons are driven by the [`crate::widget`] system via the
//! [`super::widget_bridge`].  The list box rows keep their custom
//! rendering rather than going through the standard listbox widget.

use crate::gfx_types::Keycode;

use crate::gfx_types::GameEvent;
use crate::renderer::Renderer;
use crate::sound::{AudioBackend, SoundManager};
use robin_assets::keyconfig::{KeyConfig, REAL_KEY_COUNT};
use robin_engine::sound_cache::SampleLoader;

use super::layout::{
    MenuRect, MenuTransform, align_bottom_right, dim_screen, draw_background,
    draw_screen_background, enter_modal_gpu_phase, render_text_virt_font,
};
use super::resources::{
    IngameMenuResources, MT_BTN_CANCEL, MT_BTN_DEFAULT_1, MT_BTN_DEFAULT_2, MT_BTN_OK,
    MT_BTN_USER_DEFINED, MT_STR_KEY_ALT, MT_STR_KEY_ALT_GR, MT_STR_KEY_BACKSPACE,
    MT_STR_KEY_CAPS_LOCK, MT_STR_KEY_CTRL_LEFT, MT_STR_KEY_CTRL_RIGHT, MT_STR_KEY_DOWN,
    MT_STR_KEY_END, MT_STR_KEY_ESC, MT_STR_KEY_F1, MT_STR_KEY_F2, MT_STR_KEY_F3, MT_STR_KEY_F4,
    MT_STR_KEY_F5, MT_STR_KEY_F6, MT_STR_KEY_F7, MT_STR_KEY_F8, MT_STR_KEY_F9, MT_STR_KEY_F10,
    MT_STR_KEY_F11, MT_STR_KEY_F12, MT_STR_KEY_HOME, MT_STR_KEY_INS, MT_STR_KEY_LEFT,
    MT_STR_KEY_NONE, MT_STR_KEY_NUM_0, MT_STR_KEY_NUM_1, MT_STR_KEY_NUM_2, MT_STR_KEY_NUM_3,
    MT_STR_KEY_NUM_4, MT_STR_KEY_NUM_5, MT_STR_KEY_NUM_6, MT_STR_KEY_NUM_7, MT_STR_KEY_NUM_8,
    MT_STR_KEY_NUM_9, MT_STR_KEY_NUM_CROSS, MT_STR_KEY_NUM_DASH, MT_STR_KEY_NUM_LOCK,
    MT_STR_KEY_NUM_RETURN, MT_STR_KEY_NUM_SLASH, MT_STR_KEY_NUM_STAR, MT_STR_KEY_NUM_SUP,
    MT_STR_KEY_PAGE_DOWN, MT_STR_KEY_PAGE_UP, MT_STR_KEY_PAUSE, MT_STR_KEY_PRINT,
    MT_STR_KEY_RESERVED, MT_STR_KEY_RETURN, MT_STR_KEY_RIGHT, MT_STR_KEY_SCROLL_LOCK,
    MT_STR_KEY_SHIFT_LEFT, MT_STR_KEY_SHIFT_RIGHT, MT_STR_KEY_SPACE, MT_STR_KEY_SUP,
    MT_STR_KEY_TAB, MT_STR_KEY_UP, MT_STR_SHORTCUT_00, MenuText,
};
use super::widget_bridge::{
    self, ModalCursor, ModalInputState, WIDGET_NOISY_EVENT_ACTIVATED, WIDGET_NOISY_LISTBOX,
};

/// Key config list box: `(30,10)..(450,460)`.
const LIST_RECT: MenuRect = MenuRect {
    x: 30,
    y: 10,
    w: 420,
    h: 450,
};

/// Column-split ratio: 70% of the row width is the key-name column,
/// 30% is the key-value column.
const COLUMN_SPLIT_RATIO: f32 = 0.700;
/// Fallback row height used when no list font is loaded (the shipping
/// config ships 16px bitmap list fonts; matches the pre-font value).
const FALLBACK_ROW_HEIGHT: i32 = 16;
// NOTE: the renderer has a 6-state font table (3 states × normal/alternate
// styles).  Shortcuts only sets the default/focused/selected fonts and never
// flags items as alternate, so the alternate slots fall back to the default
// font.  `list_font_native_with_style(focused, selected, alternate=false)`
// exposes the 6-state lookup with that same fallback — shortcuts always
// passes `alternate=false`, so the alternate style is structural only.
const ID_OK: u32 = 0;
const ID_DEFAULT1: u32 = 1;
const ID_DEFAULT2: u32 = 2;
const ID_USER: u32 = 3;
const ID_CANCEL: u32 = 4;

/// Display the shortcuts sub-screen.  Returns `true` on OK when any
/// binding was edited.
///
/// `active` is the currently-applied key config.  `custom` is the
/// user's personal custom bindings — the User Defined button restores
/// from this slot, and switching to a preset while dirty edits are
/// pending saves them here so they aren't lost.
#[allow(clippy::too_many_arguments)]
pub async fn show_shortcuts(
    event_pump: &mut crate::window::GameWindow,
    renderer: &mut Renderer,
    resources: &IngameMenuResources,
    cursor: Option<ModalCursor<'_>>,
    active: &mut KeyConfig,
    custom: &mut KeyConfig,
    mut sound: Option<&mut SoundManager>,
    mut audio_backend: Option<&mut dyn AudioBackend>,
    sample_loader: Option<&SampleLoader>,
) -> bool {
    let sw = renderer.screen_width() as i32;
    let sh = renderer.screen_height() as i32;
    let transform = MenuTransform::centered(sw, sh);

    let original = active.clone();
    let mut working = active.clone();
    // Tracks single-key edits since menu open OR last preset switch —
    // reset whenever a preset is loaded.
    let mut working_dirty = false;

    let (btn_w, btn_h) = resources.button_dimensions();
    let ok_label = resources.menu_text.get(MT_BTN_OK);
    let default1_label = resources.menu_text.get(MT_BTN_DEFAULT_1);
    let default2_label = resources.menu_text.get(MT_BTN_DEFAULT_2);
    let user_label = resources.menu_text.get(MT_BTN_USER_DEFINED);
    let cancel_label = resources.menu_text.get(MT_BTN_CANCEL);
    let labels: &[(&str, bool)] = &[
        (&ok_label, true),
        (&default1_label, true),
        (&default2_label, true),
        (&user_label, true),
        (&cancel_label, true),
    ];
    let menu_buttons = align_bottom_right(labels, btn_w, btn_h);

    // Build FrameWnd with the five buttons.
    let mut frame = crate::widget::FrameWnd::default();
    frame.enabled = true;
    frame.input_enabled = true;
    for (i, mb) in menu_buttons.iter().enumerate() {
        frame.add_widget_absolute(widget_bridge::make_button(
            i as u32, &mb.label, mb.x, mb.y, mb.w, mb.h,
        ));
    }

    let mut focused_row: Option<usize> = None;
    let mut rebinding_row: Option<usize> = None;
    // True when the user pressed a reserved scancode during the current
    // rebind — shows the localised "Reserved" string in place of the
    // "<Press a key>" prompt while edit mode stays open.  Cleared on
    // any non-reserved keydown (which assigns and exits edit mode) and
    // when the rebind target changes via mouse click.
    let mut reserved_overlay = false;
    // Tab cycles focus between the listbox and the button group; arrow
    // keys navigate within whichever group is active.  `None` = listbox,
    // `Some(idx)` = the button at index `idx` in `menu_buttons`.
    let mut focused_button: Option<usize> = None;
    let mut done = false;
    let mut accepted = false;
    // Pending button activation triggered by Return/Space while a button
    // is keyboard-focused.  Drained next to the click-driven activation
    // path so all five buttons share one handler.
    let mut keyboard_button_activation: Option<u32> = None;
    let mut input_state = ModalInputState::new();
    input_state.seed_mouse_from_sdl(event_pump, transform);
    let mut scroll_offset: usize = 0;

    // Row height from the default list font.
    let row_height = resources
        .list_font_with_style(false, false, false)
        .map(|f| f.height() as i32)
        .unwrap_or(FALLBACK_ROW_HEIGHT)
        .max(1);
    // Scrollbar track width from sub-picture 0 of the listbox bitmap
    // (the SLIDER_BACK_START slice).  The row width is shrunk by this
    // to leave room for the scrollbar, and all three back slices share
    // the same width.
    let scrollbar_w = resources.list_scrollbar[0]
        .map(|s| s.width)
        .unwrap_or(0)
        .max(0);
    let visible_rows = (LIST_RECT.h / row_height).max(1) as usize;
    let total_rows = REAL_KEY_COUNT as usize;
    let needs_scrollbar = total_rows > visible_rows && scrollbar_w > 0;

    while !done {
        for event in event_pump.poll_events() {
            input_state.update_from_event(&event, transform);
            match event {
                GameEvent::Quit => done = true,
                GameEvent::KeyDown { keycode, scancode } => {
                    if let Some(row) = rebinding_row {
                        // Escape's SDL scancode (41) is in the reserved
                        // set, but the GameEvent's `keycode` may be
                        // resolved through layout mapping; check both so
                        // either path lands in the reserved branch.
                        if keycode == Keycode::Escape || is_reserved_scancode(scancode) {
                            // Pressing a reserved key shows the localised
                            // "Reserved" string in the row and *stays* in
                            // edit mode for another attempt.  Surfaced via
                            // an overlay flag and a noise cue.
                            reserved_overlay = true;
                            play_rebind_noise(&mut sound, &mut audio_backend, sample_loader);
                        } else {
                            assign_key_with_conflict_resolution(&mut working, row as u16, scancode);
                            working.key_type = 1; // UserDefined
                            working_dirty = true;
                            rebinding_row = None;
                            reserved_overlay = false;
                            // Plays the listbox noisy bank's ACTIVATED slot
                            // on every successful rebind.
                            play_rebind_noise(&mut sound, &mut audio_backend, sample_loader);
                        }
                    } else {
                        match (keycode, focused_button) {
                            // Tab/Shift+Tab cycles between list and button group.
                            (Keycode::Tab, _) => {
                                focused_button = match focused_button {
                                    None => Some(0),
                                    Some(i) if i + 1 < menu_buttons.len() => Some(i + 1),
                                    Some(_) => None,
                                };
                            }
                            // While a button is focused: Left/Right cycle
                            // within the group; Return/Space activates it.
                            (Keycode::Left, Some(i)) => {
                                focused_button = Some(if i == 0 {
                                    menu_buttons.len() - 1
                                } else {
                                    i - 1
                                });
                            }
                            (Keycode::Right, Some(i)) => {
                                focused_button = Some((i + 1) % menu_buttons.len());
                            }
                            (Keycode::Return | Keycode::KpEnter | Keycode::Space, Some(i)) => {
                                keyboard_button_activation = Some(i as u32);
                            }
                            // List focus (focused_button == None): keep the
                            // pre-existing keyboard map for the listbox.
                            (Keycode::Escape, _) => done = true,
                            (Keycode::Return | Keycode::KpEnter, None) => {
                                // Return on a focused list row is equivalent
                                // to a left-click on that row: enter rebind
                                // mode.  If no row is focused yet, fall
                                // through to the OK accept path.
                                if let Some(row) = focused_row {
                                    rebinding_row = Some(row);
                                } else {
                                    accepted = true;
                                    done = true;
                                }
                            }
                            (Keycode::Up, None) => {
                                focused_row = Some(focused_row.map_or(0, |f| f.saturating_sub(1)));
                            }
                            (Keycode::Down, None) => {
                                let max = (REAL_KEY_COUNT as usize).saturating_sub(1);
                                focused_row =
                                    Some(focused_row.map(|f| (f + 1).min(max)).unwrap_or(0));
                            }
                            (Keycode::PageUp, None) => {
                                scroll_offset = scroll_offset.saturating_sub(visible_rows);
                            }
                            (Keycode::PageDown, None) => {
                                let max_scroll =
                                    (REAL_KEY_COUNT as usize).saturating_sub(visible_rows);
                                scroll_offset = (scroll_offset + visible_rows).min(max_scroll);
                            }
                            (Keycode::Home, None) => {
                                scroll_offset = 0;
                            }
                            (Keycode::End, None) => {
                                scroll_offset =
                                    (REAL_KEY_COUNT as usize).saturating_sub(visible_rows);
                            }
                            _ => {}
                        }
                    }
                }
                // List row click handling (not through widgets).
                GameEvent::MouseUp(x, y, 1) => {
                    let (vx, vy) = transform.from_screen(x, y);
                    if LIST_RECT.contains_virt(vx, vy) {
                        let row_offset = ((vy - LIST_RECT.y - 4) / row_height).max(0) as usize;
                        let row = scroll_offset + row_offset;
                        if row < total_rows {
                            focused_row = Some(row);
                            rebinding_row = Some(row);
                            reserved_overlay = false;
                        }
                    }
                }
                GameEvent::MouseWheel(dy) => {
                    let max_scroll = (REAL_KEY_COUNT as usize).saturating_sub(visible_rows);
                    if dy > 0 {
                        scroll_offset = scroll_offset.saturating_sub(1);
                    } else if dy < 0 {
                        scroll_offset = (scroll_offset + 1).min(max_scroll);
                    }
                }
                _ => {}
            }
        }

        // Widget input for buttons.
        let widget_input = input_state.as_widget_input();
        let events = frame.process_input(&widget_input);
        input_state.end_frame();

        // Both mouse-driven (find_activated) and keyboard-driven
        // (Tab+Return) activations route through the same handler.
        let activated_id = widget_bridge::find_activated(&events).or(keyboard_button_activation);
        keyboard_button_activation = None;
        if let Some(id) = activated_id {
            match id {
                ID_OK => {
                    accepted = true;
                    done = true;
                }
                ID_DEFAULT1 => {
                    let record =
                        robin_engine::engine::GlobalOptions::record_default_key_config_global();
                    set_to_preset(&mut working, custom, &mut working_dirty, 0, record);
                }
                ID_DEFAULT2 => {
                    let record =
                        robin_engine::engine::GlobalOptions::record_default_key_config_global();
                    set_to_preset(&mut working, custom, &mut working_dirty, 1, record);
                }
                ID_USER => {
                    apply_user_defined(&mut working, custom, &mut working_dirty);
                }
                ID_CANCEL => done = true,
                _ => {}
            }
        }

        enter_modal_gpu_phase(renderer);
        dim_screen(renderer);

        if let Some(bg) = resources.menu_bg[3] {
            draw_screen_background(renderer, &bg);
        }

        // List box frame.
        if let Some(list_frame) = resources.list_box {
            draw_background(
                renderer,
                transform,
                &list_frame,
                LIST_RECT.x,
                LIST_RECT.y,
                LIST_RECT.w,
                LIST_RECT.h,
            );
        } else {
            let (sx, sy) = transform.to_screen(LIST_RECT.x, LIST_RECT.y);
            renderer.fill_screen(
                Some(&robin_engine::sprite::BBox::new(
                    crate::geo2d::pt(sx as f32, sy as f32),
                    crate::geo2d::pt((sx + LIST_RECT.w) as f32, (sy + LIST_RECT.h) as f32),
                )),
                Renderer::create_color_16(30, 25, 15),
            );
            renderer.draw_rect_outline_screen(
                sx,
                sy,
                sx + LIST_RECT.w,
                sy + LIST_RECT.h,
                Renderer::create_color_16(180, 160, 100),
            );
        }

        // Key binding rows.  Each row is split into a 70% key-name column
        // (left-aligned, " : " suffix) and a 30% key-value column
        // (centered).  The row width subtracts the scrollbar gutter so
        // text doesn't run under the scrollbar.
        let gutter = if needs_scrollbar { scrollbar_w } else { 0 };
        let text_x = LIST_RECT.x + 10;
        let text_w = (LIST_RECT.w - 20 - gutter).max(0);
        let split = (text_w as f32 * COLUMN_SPLIT_RATIO) as i32;
        let key_name_x = text_x;
        let key_value_x = text_x + split;
        let key_value_w = text_w - split;
        for row_offset in 0..visible_rows {
            let row_index = scroll_offset + row_offset;
            if row_index >= total_rows {
                break;
            }
            let row_top = LIST_RECT.y + 4 + row_offset as i32 * row_height;
            let action_label = resources.menu_text.get(MT_STR_SHORTCUT_00 + row_index);
            let key_value = working.get_key_by_index(row_index as u16);
            let key_label = scancode_display_name(&resources.menu_text, key_value);

            let is_focused = focused_row == Some(row_index);
            let is_rebinding = rebinding_row == Some(row_index);
            // `alternate=false` — shortcuts never flags rows as alternate
            // (see note on the const block above).
            let font = resources.list_font_with_style(is_focused, is_rebinding, false);
            let Some(font) = font else { continue };

            // Vertically centre the glyphs inside the row box.
            let text_y = row_top + (row_height - font.height() as i32) / 2;

            // Left column: "<action> : ", left-aligned within the 70% box.
            let name_with_sep = format!("{action_label} : ");
            render_text_virt_font(
                renderer,
                font,
                transform,
                &name_with_sep,
                key_name_x,
                text_y,
            );

            // Right column: key value (or rebind prompt), centered within
            // the 30% box.  When a reserved scancode was just pressed,
            // show the localised "Reserved" string instead of the rebind
            // prompt.
            let display = if is_rebinding {
                if reserved_overlay {
                    resources.menu_text.get(MT_STR_KEY_RESERVED)
                } else {
                    "<Press a key>".to_string()
                }
            } else {
                key_label
            };
            let val_w = font.text_width(&display);
            let val_x = key_value_x + ((key_value_w - val_w) / 2).max(0);
            render_text_virt_font(renderer, font, transform, &display, val_x, text_y);
        }

        // Scrollbar on the right edge.
        if needs_scrollbar {
            draw_listbox_scrollbar(
                renderer,
                transform,
                resources,
                LIST_RECT.x + LIST_RECT.w - scrollbar_w,
                LIST_RECT.y,
                scrollbar_w,
                LIST_RECT.h,
                scroll_offset,
                visible_rows,
                total_rows,
            );
        }

        // Buttons via widget bridge.
        widget_bridge::draw_frame_buttons(renderer, resources, transform, &frame);

        // Keyboard-focus outline around the focused button (Tab cycle).
        if let Some(idx) = focused_button
            && let Some(mb) = menu_buttons.get(idx)
        {
            let (sx, sy) = transform.to_screen(mb.x, mb.y);
            let (ex, ey) = transform.to_screen(mb.x + mb.w, mb.y + mb.h);
            renderer.draw_rect_outline_screen(
                sx - 1,
                sy - 1,
                ex + 1,
                ey + 1,
                Renderer::create_color_16(255, 220, 80),
            );
        }

        if let Some(c) = &cursor {
            c.draw(renderer, transform, &input_state);
        }

        renderer.present();
        crate::window::sleep_ms(16).await;
    }

    if accepted {
        let changed = working.get_keys_array_vec() != original.get_keys_array_vec();
        if changed {
            // On OK: if the user made edits, persist them to the custom
            // slot too so a later "User Defined" click brings them back.
            if working_dirty {
                *custom = working.clone();
            }
            *active = working;
            return true;
        }
    }
    false
}

/// Apply a Default1 / Default2 click.
///
/// If the user has dirty single-key edits since the menu opened (or
/// since the last preset switch), they're first promoted into the
/// custom slot so they aren't lost.
///
/// Then:
///   * Normal mode (`record_default_key_config == false`): load the
///     hardcoded preset for `preset_idx` (0 = keyset1, 1 = keyset2).
///   * Record mode (`-RECORDDEFAULTKEYCONFIG` launch flag): write the
///     *current* working config to the corresponding `keyset{N}.cfg`
///     file.  Developer path for rebuilding shipped presets.
fn set_to_preset(
    working: &mut KeyConfig,
    custom: &mut KeyConfig,
    working_dirty: &mut bool,
    preset_idx: u16,
    record_default_key_config: bool,
) {
    if *working_dirty {
        *custom = working.clone();
    }

    if record_default_key_config {
        // Write mode: persist the current working config to keyset{N}.cfg
        // so future Default{N} clicks return to it.  The file path is
        // relative to the datadir CWD.
        let filename = match preset_idx {
            0 => "Data/Configuration/keyset1.cfg",
            1 => "Data/Configuration/keyset2.cfg",
            _ => {
                tracing::error!("set_to_preset: invalid preset_idx {preset_idx}");
                return;
            }
        };
        // Tag the saved config as the target preset (PresetBase + idx).
        let mut to_save = working.clone();
        to_save.key_type = 2 + preset_idx;
        if let Err(err) = to_save.save_to_keyset_file(std::path::Path::new(filename)) {
            tracing::error!("Failed to save preset {preset_idx} to {filename}: {err}");
        } else {
            tracing::info!("Saved working config to {filename} (preset {preset_idx})");
        }
        // Bringing the in-memory working config in line with the file
        // we just wrote — same key_type stamp, same bindings.
        *working = to_save;
    } else {
        // Read mode: attempt to load the on-disk keyset{N}.cfg first.
        // Fall back to the hardcoded preset if the file is missing or
        // unreadable, so the RECORDDEFAULTKEYCONFIG → relaunch →
        // read-back loop still works.
        let (filename, fallback): (&str, fn() -> KeyConfig) = match preset_idx {
            0 => ("Data/Configuration/keyset1.cfg", KeyConfig::default_preset),
            1 => (
                "Data/Configuration/keyset2.cfg",
                KeyConfig::alternate_preset,
            ),
            _ => {
                tracing::error!("set_to_preset: invalid preset_idx {preset_idx}");
                return;
            }
        };
        *working = match KeyConfig::load_from_keyset_file(std::path::Path::new(filename)) {
            Ok(cfg) => cfg,
            Err(err) => {
                tracing::warn!(
                    "set_to_preset: failed to read {filename} ({err}); using hardcoded preset"
                );
                fallback()
            }
        };
    }
    *working_dirty = false;
}

/// Apply a User Defined click: restore `working` from the persisted
/// `custom` slot — the user's personal bindings, not the menu's entry
/// snapshot.
fn apply_user_defined(working: &mut KeyConfig, custom: &KeyConfig, working_dirty: &mut bool) {
    *working = custom.clone();
    working.key_type = 1; // UserDefined
    *working_dirty = false;
}

/// Draw the listbox scrollbar: 3-slice track + 3-slice thumb.
///
/// We blit each 3-slice component directly each frame rather than
/// pre-composing the full-height track and thumb into cached surfaces.
/// Simpler, and makes the thumb resize for free when the item count
/// changes.
///
/// Arguments:
/// - `(track_x, track_y, track_w, track_h)`: the track's bounding box
///   (right edge of the list, full list height).
/// - `scroll_offset`: index of the topmost visible row.
/// - `visible_rows`: how many rows fit in the list area.
/// - `total_rows`: total row count (drives `before_ratio` / `knob_ratio`).
#[allow(clippy::too_many_arguments)]
fn draw_listbox_scrollbar(
    renderer: &mut Renderer,
    transform: MenuTransform,
    resources: &IngameMenuResources,
    track_x: i32,
    track_y: i32,
    track_w: i32,
    track_h: i32,
    scroll_offset: usize,
    visible_rows: usize,
    total_rows: usize,
) {
    let slices = &resources.list_scrollbar;
    let (Some(back_start), Some(back_fill), Some(back_end)) = (slices[0], slices[1], slices[2])
    else {
        return;
    };
    let (Some(thumb_start), Some(thumb_fill), Some(thumb_end)) = (slices[3], slices[4], slices[5])
    else {
        return;
    };

    // Track (3-slice vertical): top cap, tiled fill, bottom cap.
    let start_h = back_start.height.min(track_h);
    let end_h = back_end.height.min(track_h - start_h);
    let fill_y = track_y + start_h;
    let fill_h = (track_h - start_h - end_h).max(0);
    draw_background(
        renderer,
        transform,
        &back_start,
        track_x,
        track_y,
        track_w,
        start_h,
    );
    if fill_h > 0 {
        draw_background(
            renderer, transform, &back_fill, track_x, fill_y, track_w, fill_h,
        );
    }
    draw_background(
        renderer,
        transform,
        &back_end,
        track_x,
        track_y + track_h - end_h,
        track_w,
        end_h,
    );

    // Thumb (3-slice vertical).  Placed inside a `track_h - 2` range
    // with a 1px inset top/bottom.
    let total = total_rows.max(1) as f32;
    let before_ratio = (scroll_offset as f32 / total).clamp(0.0, 1.0);
    let knob_ratio = (visible_rows as f32 / total).clamp(0.0, 1.0);
    let usable = (track_h - 2).max(0) as f32;
    let thumb_top = track_y + 1 + (usable * before_ratio) as i32;
    let thumb_bot = track_y + 1 + (usable * (before_ratio + knob_ratio)) as i32;
    let thumb_h = (thumb_bot - thumb_top).max(thumb_start.height + thumb_end.height);

    let thumb_start_h = thumb_start.height.min(thumb_h);
    let thumb_end_h = thumb_end.height.min(thumb_h - thumb_start_h);
    let thumb_fill_h = (thumb_h - thumb_start_h - thumb_end_h).max(0);
    draw_background(
        renderer,
        transform,
        &thumb_start,
        track_x,
        thumb_top,
        track_w,
        thumb_start_h,
    );
    if thumb_fill_h > 0 {
        draw_background(
            renderer,
            transform,
            &thumb_fill,
            track_x,
            thumb_top + thumb_start_h,
            track_w,
            thumb_fill_h,
        );
    }
    draw_background(
        renderer,
        transform,
        &thumb_end,
        track_x,
        thumb_top + thumb_h - thumb_end_h,
        track_w,
        thumb_end_h,
    );
}

/// Bind `scancode` to `target_index`, first clearing any other slot
/// that already holds that scancode.  Without this, the same physical
/// key would silently end up bound to two actions.
fn assign_key_with_conflict_resolution(cfg: &mut KeyConfig, target_index: u16, scancode: u16) {
    let conflict = cfg.get_index_for_key(scancode);
    if conflict != 0xFFFF && conflict != target_index {
        cfg.set_key_by_index(conflict, 0);
    }
    cfg.set_key_by_index(target_index, scancode);
}

/// Play the rebind noise — the listbox noisy bank's ACTIVATED slot.
///
/// Calls `play_menu_sound` directly rather than going through the widget
/// event-pump because the rebind handler does not flow through the
/// listbox widget — it reads `KeyDown` events straight from the modal
/// loop, so there is no `UiEvent` stream to feed `play_widget_noise`.
fn play_rebind_noise(
    sound: &mut Option<&mut SoundManager>,
    audio_backend: &mut Option<&mut dyn AudioBackend>,
    sample_loader: Option<&SampleLoader>,
) {
    let Some(s) = sound.as_deref_mut() else {
        return;
    };
    let Some(b) = audio_backend.as_deref_mut() else {
        return;
    };
    let Some(loader) = sample_loader else {
        return;
    };
    let sound_id = (WIDGET_NOISY_LISTBOX << 16) + WIDGET_NOISY_EVENT_ACTIVATED;
    s.play_menu_sound(sound_id, b, loader);
}

/// SDL scancodes that the shortcuts menu refuses to assign as user
/// bindings — pressing one of these during a rebind aborts the edit
/// without changing anything.
fn is_reserved_scancode(scancode: u16) -> bool {
    const SDL_SCANCODE_PRINTSCREEN: u16 = 70;
    const SDL_SCANCODE_ESCAPE: u16 = 41;
    const SDL_SCANCODE_LGUI: u16 = 227;
    const SDL_SCANCODE_RGUI: u16 = 231;
    const SDL_SCANCODE_MENU: u16 = 118;
    matches!(
        scancode,
        SDL_SCANCODE_PRINTSCREEN
            | SDL_SCANCODE_ESCAPE
            | SDL_SCANCODE_LGUI
            | SDL_SCANCODE_RGUI
            | SDL_SCANCODE_MENU
    )
}

/// Convert an SDL scancode to its localised display name.  Named keys
/// come from the `MT_STR_KEY_*` menu-text table (so rebound keys read
/// e.g. "Espacio" in ES); printable characters are emitted directly.
///
/// Every scancode that `convertkeys::dik_to_sdl` can emit is covered, so
/// a rebind never renders as `Scan N`.  Scancodes outside the named set
/// that fall through to a synthesised label (`F13`, `Menu`, `Keypad =`,
/// extra Win keys) are intentionally hard-coded English: there is no
/// menu-text id for them and they would otherwise render as a blank
/// cell, so a readable English name is strictly better.
fn scancode_display_name(menu_text: &MenuText, scancode: u16) -> String {
    // Helper: look up the menu-text entry for a scancode that has a
    // dedicated `MT_STR_KEY_*` id.
    let mt = |id: usize| menu_text.get(id);
    match scancode {
        0 => mt(MT_STR_KEY_NONE),
        4..=29 => {
            let c = (b'a' + (scancode as u8 - 4)) as char;
            c.to_string()
        }
        30..=38 => {
            let c = (b'1' + (scancode as u8 - 30)) as char;
            c.to_string()
        }
        39 => "0".to_string(),
        40 => mt(MT_STR_KEY_RETURN),
        41 => mt(MT_STR_KEY_ESC),
        42 => mt(MT_STR_KEY_BACKSPACE),
        43 => mt(MT_STR_KEY_TAB),
        44 => mt(MT_STR_KEY_SPACE),
        45 => "-".to_string(),
        46 => "=".to_string(),
        47 => "[".to_string(),
        48 => "]".to_string(),
        49 => "\\".to_string(),
        51 => ";".to_string(),
        52 => "'".to_string(),
        53 => "`".to_string(),
        54 => ",".to_string(),
        55 => ".".to_string(),
        56 => "/".to_string(),
        57 => mt(MT_STR_KEY_CAPS_LOCK),
        58 => mt(MT_STR_KEY_F1),
        59 => mt(MT_STR_KEY_F2),
        60 => mt(MT_STR_KEY_F3),
        61 => mt(MT_STR_KEY_F4),
        62 => mt(MT_STR_KEY_F5),
        63 => mt(MT_STR_KEY_F6),
        64 => mt(MT_STR_KEY_F7),
        65 => mt(MT_STR_KEY_F8),
        66 => mt(MT_STR_KEY_F9),
        67 => mt(MT_STR_KEY_F10),
        68 => mt(MT_STR_KEY_F11),
        69 => mt(MT_STR_KEY_F12),
        70 => mt(MT_STR_KEY_PRINT),
        71 => mt(MT_STR_KEY_SCROLL_LOCK),
        72 => mt(MT_STR_KEY_PAUSE),
        73 => mt(MT_STR_KEY_INS),
        74 => mt(MT_STR_KEY_HOME),
        75 => mt(MT_STR_KEY_PAGE_UP),
        76 => mt(MT_STR_KEY_SUP),
        77 => mt(MT_STR_KEY_END),
        78 => mt(MT_STR_KEY_PAGE_DOWN),
        79 => mt(MT_STR_KEY_RIGHT),
        80 => mt(MT_STR_KEY_LEFT),
        81 => mt(MT_STR_KEY_DOWN),
        82 => mt(MT_STR_KEY_UP),
        83 => mt(MT_STR_KEY_NUM_LOCK),
        84 => mt(MT_STR_KEY_NUM_SLASH),
        85 => mt(MT_STR_KEY_NUM_STAR),
        86 => mt(MT_STR_KEY_NUM_DASH),
        87 => mt(MT_STR_KEY_NUM_CROSS),
        88 => mt(MT_STR_KEY_NUM_RETURN),
        89 => mt(MT_STR_KEY_NUM_1),
        90 => mt(MT_STR_KEY_NUM_2),
        91 => mt(MT_STR_KEY_NUM_3),
        92 => mt(MT_STR_KEY_NUM_4),
        93 => mt(MT_STR_KEY_NUM_5),
        94 => mt(MT_STR_KEY_NUM_6),
        95 => mt(MT_STR_KEY_NUM_7),
        96 => mt(MT_STR_KEY_NUM_8),
        97 => mt(MT_STR_KEY_NUM_9),
        98 => mt(MT_STR_KEY_NUM_0),
        99 => mt(MT_STR_KEY_NUM_SUP),
        103 => "Keypad =".to_string(),
        104 => "F13".to_string(),
        105 => "F14".to_string(),
        106 => "F15".to_string(),
        118 => "Menu".to_string(),
        154 => "SysReq".to_string(),
        224 => mt(MT_STR_KEY_CTRL_LEFT),
        225 => mt(MT_STR_KEY_SHIFT_LEFT),
        226 => mt(MT_STR_KEY_ALT),
        227 => "Left Win".to_string(),
        228 => mt(MT_STR_KEY_CTRL_RIGHT),
        229 => mt(MT_STR_KEY_SHIFT_RIGHT),
        230 => mt(MT_STR_KEY_ALT_GR),
        231 => "Right Win".to_string(),
        _ => format!("Scan {scancode}"),
    }
}

trait KeyConfigVec {
    fn get_keys_array_vec(&self) -> Vec<u16>;
}

impl KeyConfigVec for KeyConfig {
    fn get_keys_array_vec(&self) -> Vec<u16> {
        let mut out = vec![0u16; REAL_KEY_COUNT as usize];
        self.get_keys_array(&mut out);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_to_preset_load_mode_promotes_dirty_edits_into_custom() {
        // User's persistent custom slot (whatever they had saved before).
        let mut custom = KeyConfig::default();
        custom.set_binding("ZoomIn", 99, 0);
        let custom_before = custom.clone();

        // Working copy with an in-progress edit.
        let mut working = KeyConfig::default_preset();
        working.set_key_by_index(0, 200); // override ZoomIn
        let mut dirty = true;

        // preset_idx=1 → alternate preset; record=false → load mode.
        set_to_preset(&mut working, &mut custom, &mut dirty, 1, false);

        assert!(!dirty, "dirty flag should be cleared after preset switch");
        assert_eq!(
            working.get_key_by_index(0),
            KeyConfig::alternate_preset().get_key_by_index(0),
            "working should now hold the alternate preset"
        );
        assert_ne!(
            custom.get_keys_array_vec(),
            custom_before.get_keys_array_vec(),
            "custom should have been overwritten with the user's edits"
        );
        assert_eq!(
            custom.get_binding("ZoomIn").unwrap().primary_key,
            200,
            "custom should hold the in-progress ZoomIn edit"
        );
    }

    #[test]
    fn set_to_preset_record_mode_writes_keyset_file() {
        // Record mode: working config must be written to disk at
        // Data/Configuration/keyset{N}.cfg, and *not* replaced by the
        // hardcoded preset.  We cd into a temp dir so the relative path
        // resolves locally, matching the real game which chdirs into
        // the datadir at startup.
        let dir = tempfile::tempdir().unwrap();
        let prev_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let mut custom = KeyConfig::default();
        let mut working = KeyConfig::default_preset();
        working.set_key_by_index(0, 77); // a bespoke ZoomIn value
        let mut dirty = false;

        set_to_preset(&mut working, &mut custom, &mut dirty, 0, true);

        // Restore cwd before any assertion so a failed test doesn't
        // leave the process pointing at the temp dir.
        let keyset_path = dir.path().join("Data/Configuration/keyset1.cfg");
        std::env::set_current_dir(prev_cwd).unwrap();

        assert!(
            keyset_path.exists(),
            "record mode should have written the keyset file"
        );

        // The on-disk file must round-trip back to the working config.
        let from_disk = KeyConfig::load_from_keyset_file(&keyset_path).unwrap();
        assert_eq!(
            from_disk.get_key_by_index(0),
            77,
            "saved keyset should contain the bespoke ZoomIn value"
        );
        assert_eq!(
            from_disk.key_type, 2,
            "saved keyset should carry PresetBase + 0 as its type"
        );
        // And working must keep the bindings we just saved, not a
        // hardcoded preset.
        assert_eq!(working.get_key_by_index(0), 77);
    }

    #[test]
    fn apply_user_defined_restores_from_custom_slot() {
        // Simulate: the user has a saved custom config from a previous session.
        let mut custom = KeyConfig::default();
        custom.set_binding("ZoomIn", 111, 0);
        custom.set_binding("Action1", 222, 0);

        // Working is currently on a preset (e.g. Default1 was just clicked).
        let mut working = KeyConfig::default_preset();
        let mut dirty = false;

        apply_user_defined(&mut working, &custom, &mut dirty);

        assert!(!dirty);
        assert_eq!(working.key_type, 1, "should flag as UserDefined");
        assert_eq!(
            working.get_binding("ZoomIn").unwrap().primary_key,
            111,
            "should restore the persisted custom ZoomIn binding"
        );
        assert_eq!(
            working.get_binding("Action1").unwrap().primary_key,
            222,
            "should restore the persisted custom Action1 binding"
        );
    }

    #[test]
    fn conflict_resolution_clears_other_slot_then_assigns() {
        let mut cfg = KeyConfig::default_preset();
        // Pick a scancode currently bound to ZoomIn and try to put it
        // on a different action.  The old binding must be cleared.
        let zoom_in_idx = 0u16;
        let action1_idx = 18u16;
        let zoom_in_key = cfg.get_key_by_index(zoom_in_idx);
        assert_ne!(zoom_in_key, 0, "test fixture sanity");

        assign_key_with_conflict_resolution(&mut cfg, action1_idx, zoom_in_key);

        assert_eq!(
            cfg.get_key_by_index(action1_idx),
            zoom_in_key,
            "target slot should now hold the scancode"
        );
        assert_eq!(
            cfg.get_key_by_index(zoom_in_idx),
            0,
            "previous owner should have been cleared"
        );
    }

    #[test]
    fn conflict_resolution_no_op_on_self_assign() {
        let mut cfg = KeyConfig::default_preset();
        let action1_idx = 18u16;
        let key = cfg.get_key_by_index(action1_idx);

        // Re-binding the slot to the same scancode it already holds
        // must NOT clear it (the conflict-and-target are the same row).
        assign_key_with_conflict_resolution(&mut cfg, action1_idx, key);

        assert_eq!(cfg.get_key_by_index(action1_idx), key);
    }

    #[test]
    fn reserved_scancodes_are_blocked() {
        // Sample of every reserved scancode the menu rejects.
        for sc in [70u16, 41, 227, 231, 118] {
            assert!(is_reserved_scancode(sc), "scancode {sc} should be reserved");
        }
        // A regular letter must not be reserved.
        assert!(!is_reserved_scancode(4)); // SDL 'A'
        assert!(!is_reserved_scancode(82)); // SDL Up arrow
    }

    #[test]
    fn scancode_display_table_covers_every_convertkeys_output() {
        // Every SDL scancode `dik_to_sdl` can produce must have a
        // human-readable name (no `Scan N` fallback).  Walk every DIK
        // value and check the converted result.
        let menu_text = MenuText::english_fallbacks_only();
        let mut keys: Vec<u16> = (0u16..=0xFF).collect();
        robin_assets::convertkeys::convert_keys(&mut keys);
        for sdl in keys {
            if sdl == 0 {
                continue; // Unmapped DIK codes legitimately produce 0.
            }
            let name = scancode_display_name(&menu_text, sdl);
            assert!(
                !name.starts_with("Scan "),
                "SDL scancode {sdl} must have a friendly name (got {name})"
            );
        }
    }

    #[test]
    fn scancode_display_uses_menu_text_for_named_keys() {
        // Override the menu-text table with a sentinel translation; if the
        // lookup is wired correctly, named keys should pick up the override.
        let mut menu_text = MenuText::english_fallbacks_only();
        // Build a `strings` vector long enough to cover MT_STR_KEY_SPACE (188)
        // and inject "Espacio" (Spanish translation) at that slot.
        let mut strings: Vec<String> = vec![String::new(); MT_STR_KEY_SPACE + 1];
        strings[MT_STR_KEY_SPACE] = "Espacio".to_string();
        menu_text.replace_strings_for_test(strings);
        assert_eq!(scancode_display_name(&menu_text, 44), "Espacio");
    }

    #[test]
    fn scancode_display_falls_back_to_english_without_menu_text() {
        let menu_text = MenuText::english_fallbacks_only();
        // Space key — should fall back to the English label.
        assert_eq!(scancode_display_name(&menu_text, 44), "Space");
        // Letter key — printable character path, no menu text lookup.
        assert_eq!(scancode_display_name(&menu_text, 4), "a");
    }

    #[test]
    fn set_to_preset_clean_does_not_touch_custom() {
        let mut custom = KeyConfig::default();
        custom.set_binding("ZoomIn", 99, 0);
        let custom_before = custom.clone();

        let mut working = KeyConfig::default_preset();
        let mut dirty = false;

        set_to_preset(&mut working, &mut custom, &mut dirty, 1, false);

        assert!(!dirty);
        assert_eq!(
            custom.get_keys_array_vec(),
            custom_before.get_keys_array_vec(),
            "custom must not be overwritten when there are no pending edits"
        );
    }
}
