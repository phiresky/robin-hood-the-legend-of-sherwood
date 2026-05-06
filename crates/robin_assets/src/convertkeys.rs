//! DirectInput -> SDL scancode conversion.
//!
//! The old DirectInput conversion table used a lazily-initialized static
//! lookup table mapping DirectInput DIK_* scan codes to SDL_Scancode values.
//! This module replaces that with a pure `const fn` match — no mutable
//! statics, no lazy init.

// ---------------------------------------------------------------------------
// DirectInput key codes (DIK_*)
// ---------------------------------------------------------------------------

const DIK_ESCAPE: u16 = 0x01;
const DIK_1: u16 = 0x02;
const DIK_2: u16 = 0x03;
const DIK_3: u16 = 0x04;
const DIK_4: u16 = 0x05;
const DIK_5: u16 = 0x06;
const DIK_6: u16 = 0x07;
const DIK_7: u16 = 0x08;
const DIK_8: u16 = 0x09;
const DIK_9: u16 = 0x0A;
const DIK_0: u16 = 0x0B;
const DIK_MINUS: u16 = 0x0C;
const DIK_EQUALS: u16 = 0x0D;
const DIK_BACK: u16 = 0x0E;
const DIK_TAB: u16 = 0x0F;
const DIK_Q: u16 = 0x10;
const DIK_W: u16 = 0x11;
const DIK_E: u16 = 0x12;
const DIK_R: u16 = 0x13;
const DIK_T: u16 = 0x14;
const DIK_Y: u16 = 0x15;
const DIK_U: u16 = 0x16;
const DIK_I: u16 = 0x17;
const DIK_O: u16 = 0x18;
const DIK_P: u16 = 0x19;
const DIK_LBRACKET: u16 = 0x1A;
const DIK_RBRACKET: u16 = 0x1B;
const DIK_RETURN: u16 = 0x1C;
const DIK_LCONTROL: u16 = 0x1D;
const DIK_A: u16 = 0x1E;
const DIK_S: u16 = 0x1F;
const DIK_D: u16 = 0x20;
const DIK_F: u16 = 0x21;
const DIK_G: u16 = 0x22;
const DIK_H: u16 = 0x23;
const DIK_J: u16 = 0x24;
const DIK_K: u16 = 0x25;
const DIK_L: u16 = 0x26;
const DIK_SEMICOLON: u16 = 0x27;
const DIK_APOSTROPHE: u16 = 0x28;
const DIK_GRAVE: u16 = 0x29;
const DIK_LSHIFT: u16 = 0x2A;
const DIK_BACKSLASH: u16 = 0x2B;
const DIK_Z: u16 = 0x2C;
const DIK_X: u16 = 0x2D;
const DIK_C: u16 = 0x2E;
const DIK_V: u16 = 0x2F;
const DIK_B: u16 = 0x30;
const DIK_N: u16 = 0x31;
const DIK_M: u16 = 0x32;
const DIK_COMMA: u16 = 0x33;
const DIK_PERIOD: u16 = 0x34;
const DIK_SLASH: u16 = 0x35;
const DIK_RSHIFT: u16 = 0x36;
const DIK_MULTIPLY: u16 = 0x37;
const DIK_LMENU: u16 = 0x38;
const DIK_SPACE: u16 = 0x39;
const DIK_CAPITAL: u16 = 0x3A;
const DIK_F1: u16 = 0x3B;
const DIK_F2: u16 = 0x3C;
const DIK_F3: u16 = 0x3D;
const DIK_F4: u16 = 0x3E;
const DIK_F5: u16 = 0x3F;
const DIK_F6: u16 = 0x40;
const DIK_F7: u16 = 0x41;
const DIK_F8: u16 = 0x42;
const DIK_F9: u16 = 0x43;
const DIK_F10: u16 = 0x44;
const DIK_NUMLOCK: u16 = 0x45;
const DIK_SCROLL: u16 = 0x46;
const DIK_NUMPAD7: u16 = 0x47;
const DIK_NUMPAD8: u16 = 0x48;
const DIK_NUMPAD9: u16 = 0x49;
const DIK_SUBTRACT: u16 = 0x4A;
const DIK_NUMPAD4: u16 = 0x4B;
const DIK_NUMPAD5: u16 = 0x4C;
const DIK_NUMPAD6: u16 = 0x4D;
const DIK_ADD: u16 = 0x4E;
const DIK_NUMPAD1: u16 = 0x4F;
const DIK_NUMPAD2: u16 = 0x50;
const DIK_NUMPAD3: u16 = 0x51;
const DIK_NUMPAD0: u16 = 0x52;
const DIK_DECIMAL: u16 = 0x53;
const DIK_OEM_102: u16 = 0x56;
const DIK_F11: u16 = 0x57;
const DIK_F12: u16 = 0x58;
const DIK_F13: u16 = 0x64;
const DIK_F14: u16 = 0x65;
const DIK_F15: u16 = 0x66;
const DIK_NUMPADEQUALS: u16 = 0x8D;
const DIK_NUMPADENTER: u16 = 0x9C;
const DIK_RCONTROL: u16 = 0x9D;
const DIK_DIVIDE: u16 = 0xB5;
const DIK_SYSRQ: u16 = 0xB7;
const DIK_RMENU: u16 = 0xB8;
const DIK_PAUSE: u16 = 0xC5;
const DIK_HOME: u16 = 0xC7;
const DIK_UP: u16 = 0xC8;
const DIK_PRIOR: u16 = 0xC9;
const DIK_LEFT: u16 = 0xCB;
const DIK_RIGHT: u16 = 0xCD;
const DIK_END: u16 = 0xCF;
const DIK_DOWN: u16 = 0xD0;
const DIK_NEXT: u16 = 0xD1;
const DIK_INSERT: u16 = 0xD2;
const DIK_DELETE: u16 = 0xD3;
const DIK_LWIN: u16 = 0xDB;
const DIK_RWIN: u16 = 0xDC;
const DIK_APPS: u16 = 0xDD;

// ---------------------------------------------------------------------------
// SDL_Scancode values (from SDL_scancode.h)
// ---------------------------------------------------------------------------

const SDL_SCANCODE_A: u16 = 4;
const SDL_SCANCODE_B: u16 = 5;
const SDL_SCANCODE_C: u16 = 6;
const SDL_SCANCODE_D: u16 = 7;
const SDL_SCANCODE_E: u16 = 8;
const SDL_SCANCODE_F: u16 = 9;
const SDL_SCANCODE_G: u16 = 10;
const SDL_SCANCODE_H: u16 = 11;
const SDL_SCANCODE_I: u16 = 12;
const SDL_SCANCODE_J: u16 = 13;
const SDL_SCANCODE_K: u16 = 14;
const SDL_SCANCODE_L: u16 = 15;
const SDL_SCANCODE_M: u16 = 16;
const SDL_SCANCODE_N: u16 = 17;
const SDL_SCANCODE_O: u16 = 18;
const SDL_SCANCODE_P: u16 = 19;
const SDL_SCANCODE_Q: u16 = 20;
const SDL_SCANCODE_R: u16 = 21;
const SDL_SCANCODE_S: u16 = 22;
const SDL_SCANCODE_T: u16 = 23;
const SDL_SCANCODE_U: u16 = 24;
const SDL_SCANCODE_V: u16 = 25;
const SDL_SCANCODE_W: u16 = 26;
const SDL_SCANCODE_X: u16 = 27;
const SDL_SCANCODE_Y: u16 = 28;
const SDL_SCANCODE_Z: u16 = 29;
const SDL_SCANCODE_1: u16 = 30;
const SDL_SCANCODE_2: u16 = 31;
const SDL_SCANCODE_3: u16 = 32;
const SDL_SCANCODE_4: u16 = 33;
const SDL_SCANCODE_5: u16 = 34;
const SDL_SCANCODE_6: u16 = 35;
const SDL_SCANCODE_7: u16 = 36;
const SDL_SCANCODE_8: u16 = 37;
const SDL_SCANCODE_9: u16 = 38;
const SDL_SCANCODE_0: u16 = 39;
const SDL_SCANCODE_RETURN: u16 = 40;
const SDL_SCANCODE_ESCAPE: u16 = 41;
const SDL_SCANCODE_BACKSPACE: u16 = 42;
const SDL_SCANCODE_TAB: u16 = 43;
const SDL_SCANCODE_SPACE: u16 = 44;
const SDL_SCANCODE_MINUS: u16 = 45;
const SDL_SCANCODE_EQUALS: u16 = 46;
const SDL_SCANCODE_LEFTBRACKET: u16 = 47;
const SDL_SCANCODE_RIGHTBRACKET: u16 = 48;
const SDL_SCANCODE_BACKSLASH: u16 = 49;
const SDL_SCANCODE_SEMICOLON: u16 = 51;
const SDL_SCANCODE_APOSTROPHE: u16 = 52;
const SDL_SCANCODE_GRAVE: u16 = 53;
const SDL_SCANCODE_COMMA: u16 = 54;
const SDL_SCANCODE_PERIOD: u16 = 55;
const SDL_SCANCODE_SLASH: u16 = 56;
const SDL_SCANCODE_CAPSLOCK: u16 = 57;
const SDL_SCANCODE_F1: u16 = 58;
const SDL_SCANCODE_F2: u16 = 59;
const SDL_SCANCODE_F3: u16 = 60;
const SDL_SCANCODE_F4: u16 = 61;
const SDL_SCANCODE_F5: u16 = 62;
const SDL_SCANCODE_F6: u16 = 63;
const SDL_SCANCODE_F7: u16 = 64;
const SDL_SCANCODE_F8: u16 = 65;
const SDL_SCANCODE_F9: u16 = 66;
const SDL_SCANCODE_F10: u16 = 67;
const SDL_SCANCODE_F11: u16 = 68;
const SDL_SCANCODE_F12: u16 = 69;
const SDL_SCANCODE_SCROLLLOCK: u16 = 71;
const SDL_SCANCODE_PAUSE: u16 = 72;
const SDL_SCANCODE_INSERT: u16 = 73;
const SDL_SCANCODE_HOME: u16 = 74;
const SDL_SCANCODE_PAGEUP: u16 = 75;
const SDL_SCANCODE_DELETE: u16 = 76;
const SDL_SCANCODE_END: u16 = 77;
const SDL_SCANCODE_PAGEDOWN: u16 = 78;
const SDL_SCANCODE_RIGHT: u16 = 79;
const SDL_SCANCODE_LEFT: u16 = 80;
const SDL_SCANCODE_DOWN: u16 = 81;
const SDL_SCANCODE_UP: u16 = 82;
const SDL_SCANCODE_NUMLOCKCLEAR: u16 = 83;
const SDL_SCANCODE_KP_DIVIDE: u16 = 84;
const SDL_SCANCODE_KP_MULTIPLY: u16 = 85;
const SDL_SCANCODE_KP_MINUS: u16 = 86;
const SDL_SCANCODE_KP_PLUS: u16 = 87;
const SDL_SCANCODE_KP_ENTER: u16 = 88;
const SDL_SCANCODE_KP_1: u16 = 89;
const SDL_SCANCODE_KP_2: u16 = 90;
const SDL_SCANCODE_KP_3: u16 = 91;
const SDL_SCANCODE_KP_4: u16 = 92;
const SDL_SCANCODE_KP_5: u16 = 93;
const SDL_SCANCODE_KP_6: u16 = 94;
const SDL_SCANCODE_KP_7: u16 = 95;
const SDL_SCANCODE_KP_8: u16 = 96;
const SDL_SCANCODE_KP_9: u16 = 97;
const SDL_SCANCODE_KP_0: u16 = 98;
const SDL_SCANCODE_KP_PERIOD: u16 = 99;
const SDL_SCANCODE_KP_EQUALS: u16 = 103;
const SDL_SCANCODE_F13: u16 = 104;
const SDL_SCANCODE_F14: u16 = 105;
const SDL_SCANCODE_F15: u16 = 106;
const SDL_SCANCODE_MENU: u16 = 118;
const SDL_SCANCODE_SYSREQ: u16 = 154;
const SDL_SCANCODE_LCTRL: u16 = 224;
const SDL_SCANCODE_LSHIFT: u16 = 225;
const SDL_SCANCODE_LALT: u16 = 226;
const SDL_SCANCODE_LGUI: u16 = 227;
const SDL_SCANCODE_RCTRL: u16 = 228;
const SDL_SCANCODE_RSHIFT: u16 = 229;
const SDL_SCANCODE_RALT: u16 = 230;
const SDL_SCANCODE_RGUI: u16 = 231;

// ---------------------------------------------------------------------------
// Conversion function
// ---------------------------------------------------------------------------

/// Convert a single DirectInput scan code to its SDL_Scancode equivalent.
/// Returns 0 for unmapped codes (matching the zero-initialized C array).
const fn dik_to_sdl(dik: u16) -> u16 {
    match dik {
        DIK_ESCAPE => SDL_SCANCODE_ESCAPE,
        DIK_1 => SDL_SCANCODE_1,
        DIK_2 => SDL_SCANCODE_2,
        DIK_3 => SDL_SCANCODE_3,
        DIK_4 => SDL_SCANCODE_4,
        DIK_5 => SDL_SCANCODE_5,
        DIK_6 => SDL_SCANCODE_6,
        DIK_7 => SDL_SCANCODE_7,
        DIK_8 => SDL_SCANCODE_8,
        DIK_9 => SDL_SCANCODE_9,
        DIK_0 => SDL_SCANCODE_0,
        DIK_MINUS => SDL_SCANCODE_MINUS,
        DIK_EQUALS => SDL_SCANCODE_EQUALS,
        DIK_BACK => SDL_SCANCODE_BACKSPACE,
        DIK_TAB => SDL_SCANCODE_TAB,
        DIK_Q => SDL_SCANCODE_Q,
        DIK_W => SDL_SCANCODE_W,
        DIK_E => SDL_SCANCODE_E,
        DIK_R => SDL_SCANCODE_R,
        DIK_T => SDL_SCANCODE_T,
        DIK_Y => SDL_SCANCODE_Y,
        DIK_U => SDL_SCANCODE_U,
        DIK_I => SDL_SCANCODE_I,
        DIK_O => SDL_SCANCODE_O,
        DIK_P => SDL_SCANCODE_P,
        DIK_LBRACKET => SDL_SCANCODE_LEFTBRACKET,
        DIK_RBRACKET => SDL_SCANCODE_RIGHTBRACKET,
        DIK_RETURN => SDL_SCANCODE_RETURN,
        DIK_LCONTROL => SDL_SCANCODE_LCTRL,
        DIK_A => SDL_SCANCODE_A,
        DIK_S => SDL_SCANCODE_S,
        DIK_D => SDL_SCANCODE_D,
        DIK_F => SDL_SCANCODE_F,
        DIK_G => SDL_SCANCODE_G,
        DIK_H => SDL_SCANCODE_H,
        DIK_J => SDL_SCANCODE_J,
        DIK_K => SDL_SCANCODE_K,
        DIK_L => SDL_SCANCODE_L,
        DIK_SEMICOLON => SDL_SCANCODE_SEMICOLON,
        DIK_APOSTROPHE => SDL_SCANCODE_APOSTROPHE,
        DIK_GRAVE => SDL_SCANCODE_GRAVE,
        DIK_LSHIFT => SDL_SCANCODE_LSHIFT,
        DIK_BACKSLASH | DIK_OEM_102 => SDL_SCANCODE_BACKSLASH,
        DIK_Z => SDL_SCANCODE_Z,
        DIK_X => SDL_SCANCODE_X,
        DIK_C => SDL_SCANCODE_C,
        DIK_V => SDL_SCANCODE_V,
        DIK_B => SDL_SCANCODE_B,
        DIK_N => SDL_SCANCODE_N,
        DIK_M => SDL_SCANCODE_M,
        DIK_COMMA => SDL_SCANCODE_COMMA,
        DIK_PERIOD => SDL_SCANCODE_PERIOD,
        DIK_SLASH => SDL_SCANCODE_SLASH,
        DIK_RSHIFT => SDL_SCANCODE_RSHIFT,
        DIK_MULTIPLY => SDL_SCANCODE_KP_MULTIPLY,
        DIK_LMENU => SDL_SCANCODE_LALT,
        DIK_SPACE => SDL_SCANCODE_SPACE,
        DIK_CAPITAL => SDL_SCANCODE_CAPSLOCK,
        DIK_F1 => SDL_SCANCODE_F1,
        DIK_F2 => SDL_SCANCODE_F2,
        DIK_F3 => SDL_SCANCODE_F3,
        DIK_F4 => SDL_SCANCODE_F4,
        DIK_F5 => SDL_SCANCODE_F5,
        DIK_F6 => SDL_SCANCODE_F6,
        DIK_F7 => SDL_SCANCODE_F7,
        DIK_F8 => SDL_SCANCODE_F8,
        DIK_F9 => SDL_SCANCODE_F9,
        DIK_F10 => SDL_SCANCODE_F10,
        DIK_NUMLOCK => SDL_SCANCODE_NUMLOCKCLEAR,
        DIK_SCROLL => SDL_SCANCODE_SCROLLLOCK,
        DIK_NUMPAD7 => SDL_SCANCODE_KP_7,
        DIK_NUMPAD8 => SDL_SCANCODE_KP_8,
        DIK_NUMPAD9 => SDL_SCANCODE_KP_9,
        DIK_SUBTRACT => SDL_SCANCODE_KP_MINUS,
        DIK_NUMPAD4 => SDL_SCANCODE_KP_4,
        DIK_NUMPAD5 => SDL_SCANCODE_KP_5,
        DIK_NUMPAD6 => SDL_SCANCODE_KP_6,
        DIK_ADD => SDL_SCANCODE_KP_PLUS,
        DIK_NUMPAD1 => SDL_SCANCODE_KP_1,
        DIK_NUMPAD2 => SDL_SCANCODE_KP_2,
        DIK_NUMPAD3 => SDL_SCANCODE_KP_3,
        DIK_NUMPAD0 => SDL_SCANCODE_KP_0,
        DIK_DECIMAL => SDL_SCANCODE_KP_PERIOD,
        DIK_F11 => SDL_SCANCODE_F11,
        DIK_F12 => SDL_SCANCODE_F12,
        DIK_F13 => SDL_SCANCODE_F13,
        DIK_F14 => SDL_SCANCODE_F14,
        DIK_F15 => SDL_SCANCODE_F15,
        DIK_NUMPADEQUALS => SDL_SCANCODE_KP_EQUALS,
        DIK_NUMPADENTER => SDL_SCANCODE_KP_ENTER,
        DIK_RCONTROL => SDL_SCANCODE_RCTRL,
        DIK_DIVIDE => SDL_SCANCODE_KP_DIVIDE,
        DIK_SYSRQ => SDL_SCANCODE_SYSREQ,
        DIK_RMENU => SDL_SCANCODE_RALT,
        DIK_PAUSE => SDL_SCANCODE_PAUSE,
        DIK_HOME => SDL_SCANCODE_HOME,
        DIK_UP => SDL_SCANCODE_UP,
        DIK_PRIOR => SDL_SCANCODE_PAGEUP,
        DIK_LEFT => SDL_SCANCODE_LEFT,
        DIK_RIGHT => SDL_SCANCODE_RIGHT,
        DIK_END => SDL_SCANCODE_END,
        DIK_DOWN => SDL_SCANCODE_DOWN,
        DIK_NEXT => SDL_SCANCODE_PAGEDOWN,
        DIK_INSERT => SDL_SCANCODE_INSERT,
        DIK_DELETE => SDL_SCANCODE_DELETE,
        DIK_LWIN => SDL_SCANCODE_LGUI,
        DIK_RWIN => SDL_SCANCODE_RGUI,
        DIK_APPS => SDL_SCANCODE_MENU,
        _ => 0,
    }
}

/// Convert an array of DirectInput key codes to SDL scancodes in place.
pub fn convert_keys(keys: &mut [u16]) {
    for key in keys.iter_mut() {
        *key = dik_to_sdl(*key);
    }
}

/// Inverse of [`dik_to_sdl`].  Used when writing an SDL-encoded
/// `KeyConfig` back to a legacy `keyset*.cfg` file (the
/// `gGlobalOptions.bRecordDefaultKeyConfig` write-back path mirrored by
/// `RHKeyConfig::SetToPreset(idx, true)`).
///
/// `DIK_BACKSLASH` and `DIK_OEM_102` both map to `SDL_SCANCODE_BACKSLASH`
/// in the forward table; this inverse picks the canonical
/// `DIK_BACKSLASH`.  Returns `0` for SDL scancodes the original game
/// can't represent (matches the all-zero default `convert_keys` produces
/// for unmapped DIK codes).
pub const fn sdl_to_dik(sdl: u16) -> u16 {
    match sdl {
        SDL_SCANCODE_A => DIK_A,
        SDL_SCANCODE_B => DIK_B,
        SDL_SCANCODE_C => DIK_C,
        SDL_SCANCODE_D => DIK_D,
        SDL_SCANCODE_E => DIK_E,
        SDL_SCANCODE_F => DIK_F,
        SDL_SCANCODE_G => DIK_G,
        SDL_SCANCODE_H => DIK_H,
        SDL_SCANCODE_I => DIK_I,
        SDL_SCANCODE_J => DIK_J,
        SDL_SCANCODE_K => DIK_K,
        SDL_SCANCODE_L => DIK_L,
        SDL_SCANCODE_M => DIK_M,
        SDL_SCANCODE_N => DIK_N,
        SDL_SCANCODE_O => DIK_O,
        SDL_SCANCODE_P => DIK_P,
        SDL_SCANCODE_Q => DIK_Q,
        SDL_SCANCODE_R => DIK_R,
        SDL_SCANCODE_S => DIK_S,
        SDL_SCANCODE_T => DIK_T,
        SDL_SCANCODE_U => DIK_U,
        SDL_SCANCODE_V => DIK_V,
        SDL_SCANCODE_W => DIK_W,
        SDL_SCANCODE_X => DIK_X,
        SDL_SCANCODE_Y => DIK_Y,
        SDL_SCANCODE_Z => DIK_Z,
        SDL_SCANCODE_1 => DIK_1,
        SDL_SCANCODE_2 => DIK_2,
        SDL_SCANCODE_3 => DIK_3,
        SDL_SCANCODE_4 => DIK_4,
        SDL_SCANCODE_5 => DIK_5,
        SDL_SCANCODE_6 => DIK_6,
        SDL_SCANCODE_7 => DIK_7,
        SDL_SCANCODE_8 => DIK_8,
        SDL_SCANCODE_9 => DIK_9,
        SDL_SCANCODE_0 => DIK_0,
        SDL_SCANCODE_RETURN => DIK_RETURN,
        SDL_SCANCODE_ESCAPE => DIK_ESCAPE,
        SDL_SCANCODE_BACKSPACE => DIK_BACK,
        SDL_SCANCODE_TAB => DIK_TAB,
        SDL_SCANCODE_SPACE => DIK_SPACE,
        SDL_SCANCODE_MINUS => DIK_MINUS,
        SDL_SCANCODE_EQUALS => DIK_EQUALS,
        SDL_SCANCODE_LEFTBRACKET => DIK_LBRACKET,
        SDL_SCANCODE_RIGHTBRACKET => DIK_RBRACKET,
        SDL_SCANCODE_BACKSLASH => DIK_BACKSLASH,
        SDL_SCANCODE_SEMICOLON => DIK_SEMICOLON,
        SDL_SCANCODE_APOSTROPHE => DIK_APOSTROPHE,
        SDL_SCANCODE_GRAVE => DIK_GRAVE,
        SDL_SCANCODE_COMMA => DIK_COMMA,
        SDL_SCANCODE_PERIOD => DIK_PERIOD,
        SDL_SCANCODE_SLASH => DIK_SLASH,
        SDL_SCANCODE_CAPSLOCK => DIK_CAPITAL,
        SDL_SCANCODE_F1 => DIK_F1,
        SDL_SCANCODE_F2 => DIK_F2,
        SDL_SCANCODE_F3 => DIK_F3,
        SDL_SCANCODE_F4 => DIK_F4,
        SDL_SCANCODE_F5 => DIK_F5,
        SDL_SCANCODE_F6 => DIK_F6,
        SDL_SCANCODE_F7 => DIK_F7,
        SDL_SCANCODE_F8 => DIK_F8,
        SDL_SCANCODE_F9 => DIK_F9,
        SDL_SCANCODE_F10 => DIK_F10,
        SDL_SCANCODE_F11 => DIK_F11,
        SDL_SCANCODE_F12 => DIK_F12,
        SDL_SCANCODE_F13 => DIK_F13,
        SDL_SCANCODE_F14 => DIK_F14,
        SDL_SCANCODE_F15 => DIK_F15,
        SDL_SCANCODE_NUMLOCKCLEAR => DIK_NUMLOCK,
        SDL_SCANCODE_SCROLLLOCK => DIK_SCROLL,
        SDL_SCANCODE_KP_DIVIDE => DIK_DIVIDE,
        SDL_SCANCODE_KP_MULTIPLY => DIK_MULTIPLY,
        SDL_SCANCODE_KP_MINUS => DIK_SUBTRACT,
        SDL_SCANCODE_KP_PLUS => DIK_ADD,
        SDL_SCANCODE_KP_ENTER => DIK_NUMPADENTER,
        SDL_SCANCODE_KP_1 => DIK_NUMPAD1,
        SDL_SCANCODE_KP_2 => DIK_NUMPAD2,
        SDL_SCANCODE_KP_3 => DIK_NUMPAD3,
        SDL_SCANCODE_KP_4 => DIK_NUMPAD4,
        SDL_SCANCODE_KP_5 => DIK_NUMPAD5,
        SDL_SCANCODE_KP_6 => DIK_NUMPAD6,
        SDL_SCANCODE_KP_7 => DIK_NUMPAD7,
        SDL_SCANCODE_KP_8 => DIK_NUMPAD8,
        SDL_SCANCODE_KP_9 => DIK_NUMPAD9,
        SDL_SCANCODE_KP_0 => DIK_NUMPAD0,
        SDL_SCANCODE_KP_PERIOD => DIK_DECIMAL,
        SDL_SCANCODE_KP_EQUALS => DIK_NUMPADEQUALS,
        SDL_SCANCODE_PAUSE => DIK_PAUSE,
        SDL_SCANCODE_HOME => DIK_HOME,
        SDL_SCANCODE_END => DIK_END,
        SDL_SCANCODE_PAGEUP => DIK_PRIOR,
        SDL_SCANCODE_PAGEDOWN => DIK_NEXT,
        SDL_SCANCODE_INSERT => DIK_INSERT,
        SDL_SCANCODE_DELETE => DIK_DELETE,
        SDL_SCANCODE_UP => DIK_UP,
        SDL_SCANCODE_DOWN => DIK_DOWN,
        SDL_SCANCODE_LEFT => DIK_LEFT,
        SDL_SCANCODE_RIGHT => DIK_RIGHT,
        SDL_SCANCODE_LCTRL => DIK_LCONTROL,
        SDL_SCANCODE_LSHIFT => DIK_LSHIFT,
        SDL_SCANCODE_LALT => DIK_LMENU,
        SDL_SCANCODE_LGUI => DIK_LWIN,
        SDL_SCANCODE_RCTRL => DIK_RCONTROL,
        SDL_SCANCODE_RSHIFT => DIK_RSHIFT,
        SDL_SCANCODE_RALT => DIK_RMENU,
        SDL_SCANCODE_RGUI => DIK_RWIN,
        SDL_SCANCODE_MENU => DIK_APPS,
        SDL_SCANCODE_SYSREQ => DIK_SYSRQ,
        _ => 0,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape() {
        assert_eq!(dik_to_sdl(DIK_ESCAPE), SDL_SCANCODE_ESCAPE);
        assert_eq!(dik_to_sdl(0x01), 41);
    }

    #[test]
    fn test_space() {
        assert_eq!(dik_to_sdl(DIK_SPACE), SDL_SCANCODE_SPACE);
        assert_eq!(dik_to_sdl(0x39), 44);
    }

    #[test]
    fn test_letter_a() {
        assert_eq!(dik_to_sdl(DIK_A), SDL_SCANCODE_A);
        assert_eq!(dik_to_sdl(0x1E), 4);
    }

    #[test]
    fn test_letter_z() {
        assert_eq!(dik_to_sdl(DIK_Z), SDL_SCANCODE_Z);
        assert_eq!(dik_to_sdl(0x2C), 29);
    }

    #[test]
    fn test_return() {
        assert_eq!(dik_to_sdl(DIK_RETURN), SDL_SCANCODE_RETURN);
    }

    #[test]
    fn test_f1_through_f12() {
        assert_eq!(dik_to_sdl(DIK_F1), SDL_SCANCODE_F1);
        assert_eq!(dik_to_sdl(DIK_F12), SDL_SCANCODE_F12);
    }

    #[test]
    fn test_arrow_keys() {
        assert_eq!(dik_to_sdl(DIK_UP), SDL_SCANCODE_UP);
        assert_eq!(dik_to_sdl(DIK_DOWN), SDL_SCANCODE_DOWN);
        assert_eq!(dik_to_sdl(DIK_LEFT), SDL_SCANCODE_LEFT);
        assert_eq!(dik_to_sdl(DIK_RIGHT), SDL_SCANCODE_RIGHT);
    }

    #[test]
    fn test_modifiers() {
        assert_eq!(dik_to_sdl(DIK_LSHIFT), SDL_SCANCODE_LSHIFT);
        assert_eq!(dik_to_sdl(DIK_RSHIFT), SDL_SCANCODE_RSHIFT);
        assert_eq!(dik_to_sdl(DIK_LCONTROL), SDL_SCANCODE_LCTRL);
        assert_eq!(dik_to_sdl(DIK_RCONTROL), SDL_SCANCODE_RCTRL);
        assert_eq!(dik_to_sdl(DIK_LMENU), SDL_SCANCODE_LALT);
        assert_eq!(dik_to_sdl(DIK_RMENU), SDL_SCANCODE_RALT);
    }

    #[test]
    fn test_numpad() {
        assert_eq!(dik_to_sdl(DIK_NUMPAD0), SDL_SCANCODE_KP_0);
        assert_eq!(dik_to_sdl(DIK_NUMPAD9), SDL_SCANCODE_KP_9);
        assert_eq!(dik_to_sdl(DIK_NUMPADENTER), SDL_SCANCODE_KP_ENTER);
    }

    #[test]
    fn test_oem_102_maps_to_backslash() {
        // Both DIK_BACKSLASH and DIK_OEM_102 map to SDL_SCANCODE_BACKSLASH
        assert_eq!(dik_to_sdl(DIK_BACKSLASH), SDL_SCANCODE_BACKSLASH);
        assert_eq!(dik_to_sdl(DIK_OEM_102), SDL_SCANCODE_BACKSLASH);
    }

    #[test]
    fn test_unmapped_returns_zero() {
        assert_eq!(dik_to_sdl(0x00), 0);
        assert_eq!(dik_to_sdl(0xFF), 0);
    }

    #[test]
    fn test_convert_keys_array() {
        let mut keys: Vec<u16> = vec![DIK_ESCAPE, DIK_SPACE, DIK_A, DIK_F1];
        convert_keys(&mut keys);
        assert_eq!(
            keys,
            vec![
                SDL_SCANCODE_ESCAPE,
                SDL_SCANCODE_SPACE,
                SDL_SCANCODE_A,
                SDL_SCANCODE_F1,
            ]
        );
    }
}
