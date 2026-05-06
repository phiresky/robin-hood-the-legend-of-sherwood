//! Keyboard binding configuration.
//!
//! Stores named action strings with primary and secondary key slots, and
//! provides a hardcoded default preset matching the original game's
//! `Data/Configuration/keyset1.cfg` (after DIK-to-SDL conversion via
//! [`convert_keys`]).

/// A single action‐to‐key mapping.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct KeyBinding {
    pub action: String,
    pub primary_key: u16,
    pub secondary_key: u16,
}

/// The full set of key bindings.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct KeyConfig {
    pub bindings: Vec<KeyBinding>,
    /// Config type: `0` = Unknown, `1` = UserDefined, `2+` = PresetBase+index.
    pub key_type: u16,
}

// ── Index‐to‐action‐name mapping ──

/// Action names indexed 0..27.  Index 28 is `Dummy` (the sentinel — excluded
/// from [`REAL_KEY_COUNT`]).
const KEY_NAMES: &[&str] = &[
    "ZoomIn",
    "ZoomOut",
    "ScrollUp",
    "ScrollDown",
    "ScrollLeft",
    "ScrollRight",
    "Minimap",
    "Character1",
    "Character2",
    "Character3",
    "Character4",
    "Character5",
    "AllCharacters",
    "NoneCharacters",
    "Crouch",
    "StandUp",
    "GoBehindBuildings",
    "ToggleOutlineDisplay",
    "Action1",
    "Action2",
    "Action3",
    "MoveDuringAction",
    "RecordQuickAction",
    "StartQuickAction",
    "DeleteQuickAction",
    "ShowViewCone",
    "QuickSave1",
    "QuickLoad1",
    "Dummy",
];

/// Number of real key bindings (excludes the Dummy sentinel).
pub const REAL_KEY_COUNT: u16 = (KEY_NAMES.len() - 1) as u16;
/// Total key name count including the Dummy sentinel.
pub const KEY_NAME_COUNT: u16 = KEY_NAMES.len() as u16;

impl KeyConfig {
    /// Insert or update a binding for `action`.
    pub fn set_binding(&mut self, action: &str, primary: u16, secondary: u16) {
        if let Some(b) = self.bindings.iter_mut().find(|b| b.action == action) {
            b.primary_key = primary;
            b.secondary_key = secondary;
        } else {
            self.bindings.push(KeyBinding {
                action: action.to_owned(),
                primary_key: primary,
                secondary_key: secondary,
            });
        }
    }

    /// Look up a binding by action name.
    pub fn get_binding(&self, action: &str) -> Option<&KeyBinding> {
        self.bindings.iter().find(|b| b.action == action)
    }

    /// Return the action name whose primary *or* secondary key matches `key`.
    pub fn get_action_for_key(&self, key: u16) -> Option<&str> {
        self.bindings
            .iter()
            .find(|b| b.primary_key == key || b.secondary_key == key)
            .map(|b| b.action.as_str())
    }

    // ── Index-based access ──

    /// Get the primary key for the binding at the given action index.
    /// Returns 0 if the index is out of range or the binding doesn't exist.
    pub fn get_key_by_index(&self, index: u16) -> u16 {
        KEY_NAMES
            .get(index as usize)
            .and_then(|name| self.get_binding(name))
            .map_or(0, |b| b.primary_key)
    }

    /// Set the primary key for the binding at the given action index.
    /// Preserves the existing secondary key if a binding already exists.
    pub fn set_key_by_index(&mut self, index: u16, key: u16) {
        if let Some(&name) = KEY_NAMES.get(index as usize) {
            let secondary = self.get_binding(name).map_or(0, |b| b.secondary_key);
            self.set_binding(name, key, secondary);
        }
    }

    /// Reverse lookup: find the action index whose primary key matches `key`.
    /// Returns 0xFFFF if not found.
    pub fn get_index_for_key(&self, key: u16) -> u16 {
        for (i, &name) in KEY_NAMES.iter().enumerate().take(REAL_KEY_COUNT as usize) {
            if let Some(b) = self.get_binding(name)
                && b.primary_key == key
            {
                return i as u16;
            }
        }
        0xFFFF
    }

    /// Copy all primary keys into a flat array, indexed by action.
    /// Fills up to `len` entries; missing bindings produce 0.
    pub fn get_keys_array(&self, out: &mut [u16]) {
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = KEY_NAMES
                .get(i)
                .and_then(|name| self.get_binding(name))
                .map_or(0, |b| b.primary_key);
        }
    }

    /// Load all primary keys from a flat array. Clears existing bindings and
    /// recreates them from the array.
    pub fn load_keys_array(&mut self, keys: &[u16]) {
        self.bindings.clear();
        for (i, &key) in keys.iter().enumerate() {
            if let Some(&name) = KEY_NAMES.get(i) {
                self.bindings.push(KeyBinding {
                    action: name.to_owned(),
                    primary_key: key,
                    secondary_key: 0,
                });
            }
        }
    }

    /// Returns a `KeyConfig` with the default keyset1 preset.
    ///
    /// Matches the original game's `Data/Configuration/keyset1.cfg` after
    /// DIK-to-SDL conversion (verified byte-for-byte by
    /// `keyset1_preset_matches_shipped_file`).  Used as the seed for new
    /// profiles and the Default1 button.
    pub fn default_preset() -> Self {
        // SDL scancode values, converted from the DIK codes embedded in
        // the shipped keyset1.cfg via `convertkeys::dik_to_sdl`.
        const DEFAULT_KEYS: [u16; REAL_KEY_COUNT as usize] = [
            87,  // ZoomIn               — Keypad +
            86,  // ZoomOut              — Keypad -
            82,  // ScrollUp             — Up Arrow
            81,  // ScrollDown           — Down Arrow
            80,  // ScrollLeft           — Left Arrow
            79,  // ScrollRight          — Right Arrow
            51,  // Minimap              — Semicolon
            30,  // Character1           — 1
            31,  // Character2           — 2
            32,  // Character3           — 3
            33,  // Character4           — 4
            34,  // Character5           — 5
            20,  // AllCharacters        — Q
            7,   // NoneCharacters       — D
            6,   // Crouch               — C
            22,  // StandUp              — S
            225, // GoBehindBuildings    — Left Shift
            57,  // ToggleOutlineDisplay — Caps Lock
            10,  // Action1              — G
            11,  // Action2              — H
            13,  // Action3              — J
            224, // MoveDuringAction     — Left Ctrl
            4,   // RecordQuickAction    — A
            44,  // StartQuickAction     — Space
            42,  // DeleteQuickAction    — Backspace
            226, // ShowViewCone         — Left Alt
            58,  // QuickSave1           — F1
            62,  // QuickLoad1           — F5
        ];

        let mut cfg = Self::default();
        cfg.load_keys_array(&DEFAULT_KEYS);
        cfg.key_type = 2; // PresetBase + 0
        cfg
    }

    /// Returns a `KeyConfig` with the alternate keyset2 preset.
    ///
    /// Matches the original game's `Data/Configuration/keyset2.cfg` after
    /// DIK-to-SDL conversion — the "numpad-centric" layout selected by the
    /// Default2 button (verified byte-for-byte by
    /// `keyset2_preset_matches_shipped_file`).
    pub fn alternate_preset() -> Self {
        const ALTERNATE_KEYS: [u16; REAL_KEY_COUNT as usize] = [
            87,  // ZoomIn               — Keypad +
            86,  // ZoomOut              — Keypad -
            82,  // ScrollUp             — Up Arrow
            81,  // ScrollDown           — Down Arrow
            80,  // ScrollLeft           — Left Arrow
            79,  // ScrollRight          — Right Arrow
            85,  // Minimap              — Keypad *
            89,  // Character1           — Keypad 1
            90,  // Character2           — Keypad 2
            91,  // Character3           — Keypad 3
            92,  // Character4           — Keypad 4
            93,  // Character5           — Keypad 5
            94,  // AllCharacters        — Keypad 6
            98,  // NoneCharacters       — Keypad 0
            78,  // Crouch               — Page Down
            75,  // StandUp              — Page Up
            229, // GoBehindBuildings    — Right Shift
            57,  // ToggleOutlineDisplay — Caps Lock
            95,  // Action1              — Keypad 7
            96,  // Action2              — Keypad 8
            97,  // Action3              — Keypad 9
            228, // MoveDuringAction     — Right Ctrl
            40,  // RecordQuickAction    — Return
            44,  // StartQuickAction     — Space
            42,  // DeleteQuickAction    — Backspace
            230, // ShowViewCone         — Right Alt
            58,  // QuickSave1           — F1
            62,  // QuickLoad1           — F5
        ];

        let mut cfg = Self::default();
        cfg.load_keys_array(&ALTERNATE_KEYS);
        cfg.key_type = 3; // PresetBase + 1
        cfg
    }

    /// Load key bindings from a legacy-format keyset file (e.g. `keyset1.cfg`).
    ///
    /// Binary layout:
    ///   - 16 bytes: MD5 of `"RHKeyConfig"` written as a stream fingerprint.
    ///   - 2 bytes: `u16` config type (`PresetBase + idx`).
    ///   - `KEY_NAME_COUNT * 2` bytes: `u16` DirectInput DIK scancodes,
    ///     converted to SDL scancodes via [`convert_keys`].
    ///
    /// Returns `Ok(_)` on success or `Err` if the file is unreadable or
    /// shorter than the expected layout.
    pub fn load_from_keyset_file(path: &std::path::Path) -> Result<Self, String> {
        use crate::convertkeys::convert_keys;

        let data = std::fs::read(path)
            .map_err(|e| format!("Failed to read keyset file {:?}: {}", path, e))?;

        let entry_size = std::mem::size_of::<u16>();
        let required = VALIDATE_STREAM_HEADER + entry_size * (KEY_NAME_COUNT as usize + 1);

        if data.len() < required {
            return Err(format!(
                "Keyset file too short: {} bytes, need {}",
                data.len(),
                required
            ));
        }

        // Skip the 16-byte ValidateStream MD5 fingerprint, then read type.
        let type_off = VALIDATE_STREAM_HEADER;
        let key_type = u16::from_le_bytes([data[type_off], data[type_off + 1]]);

        let mut keys = [0u16; KEY_NAME_COUNT as usize];
        for (i, slot) in keys.iter_mut().enumerate() {
            let off = type_off + entry_size + i * entry_size;
            *slot = u16::from_le_bytes([data[off], data[off + 1]]);
        }
        convert_keys(&mut keys);

        let mut cfg = Self::default();
        cfg.load_keys_array(&keys);
        cfg.key_type = key_type;
        Ok(cfg)
    }

    /// Write the current bindings to a legacy-format keyset file in the same
    /// binary layout that [`Self::load_from_keyset_file`] reads.
    ///
    /// Keys are written as DIK scancodes (the inverse of `convert_keys`)
    /// so the file round-trips with the original game's tooling.
    pub fn save_to_keyset_file(&self, path: &std::path::Path) -> Result<(), String> {
        use crate::convertkeys::sdl_to_dik;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create {:?}: {}", parent, e))?;
        }

        let entry_size = std::mem::size_of::<u16>();
        let mut out =
            Vec::with_capacity(VALIDATE_STREAM_HEADER + entry_size * (KEY_NAME_COUNT as usize + 1));

        // ValidateStream header = MD5("RHKeyConfig").
        out.extend_from_slice(&validate_stream_fingerprint(b"RHKeyConfig"));

        // Type, then DIK-encoded scancodes.
        out.extend_from_slice(&self.key_type.to_le_bytes());

        let mut sdl_keys = vec![0u16; KEY_NAME_COUNT as usize];
        self.get_keys_array(&mut sdl_keys);
        for sdl in &sdl_keys {
            let dik = sdl_to_dik(*sdl);
            out.extend_from_slice(&dik.to_le_bytes());
        }

        std::fs::write(path, &out)
            .map_err(|e| format!("Failed to write keyset file {:?}: {}", path, e))
    }
}

/// Length of the `Toolbox::ValidateStream` MD5 fingerprint that prefixes
/// every legacy-binary keyset file.
const VALIDATE_STREAM_HEADER: usize = 16;

/// Compute the 16-byte MD5 fingerprint that `Toolbox::ValidateStream`
/// writes for a given C-string identifier (e.g. `"RHKeyConfig"`).
fn validate_stream_fingerprint(ident: &[u8]) -> [u8; 16] {
    use md5_crate::{Digest, Md5};
    let mut out = [0u8; 16];
    out.copy_from_slice(Md5::digest(ident).as_slice());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get_binding() {
        let mut cfg = KeyConfig::default();
        cfg.set_binding("ZoomIn", 0x49, 0x00);
        let b = cfg.get_binding("ZoomIn").unwrap();
        assert_eq!(b.primary_key, 0x49);
        assert_eq!(b.secondary_key, 0x00);
    }

    #[test]
    fn update_existing_binding() {
        let mut cfg = KeyConfig::default();
        cfg.set_binding("ZoomIn", 0x49, 0x00);
        cfg.set_binding("ZoomIn", 0x4A, 0x4B);
        assert_eq!(cfg.bindings.len(), 1);
        let b = cfg.get_binding("ZoomIn").unwrap();
        assert_eq!(b.primary_key, 0x4A);
        assert_eq!(b.secondary_key, 0x4B);
    }

    #[test]
    fn get_binding_missing() {
        let cfg = KeyConfig::default();
        assert!(cfg.get_binding("NonExistent").is_none());
    }

    #[test]
    fn get_action_for_primary_key() {
        let mut cfg = KeyConfig::default();
        cfg.set_binding("ScrollUp", 0xC8, 0x00);
        assert_eq!(cfg.get_action_for_key(0xC8), Some("ScrollUp"));
    }

    #[test]
    fn get_action_for_secondary_key() {
        let mut cfg = KeyConfig::default();
        cfg.set_binding("ScrollUp", 0xC8, 0x57);
        assert_eq!(cfg.get_action_for_key(0x57), Some("ScrollUp"));
    }

    #[test]
    fn get_action_for_key_missing() {
        let cfg = KeyConfig::default();
        assert!(cfg.get_action_for_key(0xFF).is_none());
    }

    #[test]
    fn default_and_alternate_presets_differ() {
        let default = KeyConfig::default_preset();
        let alternate = KeyConfig::alternate_preset();

        let mut default_keys = vec![0u16; REAL_KEY_COUNT as usize];
        let mut alt_keys = vec![0u16; REAL_KEY_COUNT as usize];
        default.get_keys_array(&mut default_keys);
        alternate.get_keys_array(&mut alt_keys);

        assert_ne!(
            default_keys, alt_keys,
            "Default1 and Default2 must produce different bindings"
        );
        assert_eq!(
            default.key_type, 2,
            "default_preset key_type = PresetBase+0"
        );
        assert_eq!(
            alternate.key_type, 3,
            "alternate_preset key_type = PresetBase+1"
        );
    }

    /// Bytes of the shipped `Data/Configuration/keyset1.cfg` from the
    /// demo datadir.  Embedded as a fixture so the parser fix is
    /// regression-tested without needing the data tree on disk.
    const KEYSET1_FIXTURE: [u8; 76] = [
        0xee, 0x9f, 0x90, 0xea, 0x50, 0x36, 0xd5, 0xc6, 0x3d, 0x4d, 0xbb, 0x30, 0xe4, 0x0d, 0x7b,
        0xe0, // MD5("RHKeyConfig")
        0x02, 0x00, // type = PresetBase + 0
        0x4e, 0x00, 0x4a, 0x00, 0xc8, 0x00, 0xd0, 0x00, 0xcb, 0x00, 0xcd, 0x00, 0x27, 0x00, 0x02,
        0x00, 0x03, 0x00, 0x04, 0x00, 0x05, 0x00, 0x06, 0x00, 0x10, 0x00, 0x20, 0x00, 0x2e, 0x00,
        0x1f, 0x00, 0x2a, 0x00, 0x3a, 0x00, 0x22, 0x00, 0x23, 0x00, 0x24, 0x00, 0x1d, 0x00, 0x1e,
        0x00, 0x39, 0x00, 0x0e, 0x00, 0x38, 0x00, 0x3b, 0x00, 0x3f, 0x00, 0x3f, 0x00,
    ];

    #[test]
    fn load_from_keyset_file_skips_md5_header() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("keyset1.cfg");
        std::fs::write(&path, KEYSET1_FIXTURE).unwrap();

        let cfg = KeyConfig::load_from_keyset_file(&path).unwrap();

        assert_eq!(cfg.key_type, 2, "type field is PresetBase + 0");
        // First DIK is 0x4E (DIK_ADD) → SDL KP_PLUS = 87.  This proves
        // we read past the 16-byte MD5 header instead of treating the
        // hash as the type / first key.
        assert_eq!(
            cfg.get_binding("ZoomIn").unwrap().primary_key,
            87,
            "ZoomIn should decode to Numpad + (SDL 87)"
        );
        assert_eq!(
            cfg.get_binding("Minimap").unwrap().primary_key,
            51,
            "Minimap should decode to Semicolon (SDL 51)"
        );
    }

    #[test]
    fn save_then_load_roundtrips_through_keyset_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("custom.cfg");
        let original = KeyConfig::default_preset();
        original.save_to_keyset_file(&path).unwrap();

        let loaded = KeyConfig::load_from_keyset_file(&path).unwrap();
        let mut a = vec![0u16; REAL_KEY_COUNT as usize];
        let mut b = vec![0u16; REAL_KEY_COUNT as usize];
        original.get_keys_array(&mut a);
        loaded.get_keys_array(&mut b);
        assert_eq!(a, b, "save/load must round-trip every binding");
        assert_eq!(loaded.key_type, original.key_type);
    }

    #[test]
    fn save_to_keyset_matches_original_byte_layout() {
        // Loading the fixture and saving it again must reproduce the
        // exact bytes — proves the writer is the inverse of the reader.
        let dir = tempfile::tempdir().unwrap();
        let in_path = dir.path().join("in.cfg");
        let out_path = dir.path().join("out.cfg");
        std::fs::write(&in_path, KEYSET1_FIXTURE).unwrap();

        let cfg = KeyConfig::load_from_keyset_file(&in_path).unwrap();
        cfg.save_to_keyset_file(&out_path).unwrap();

        let written = std::fs::read(&out_path).unwrap();
        assert_eq!(
            &written[..],
            &KEYSET1_FIXTURE[..],
            "round-tripped bytes must match the original keyset1.cfg"
        );
    }

    /// Bytes of the shipped `Data/Configuration/keyset2.cfg` from the
    /// fullgame datadir — identical to the demo_leicester_ecoste copy.
    const KEYSET2_FIXTURE: [u8; 76] = [
        0xee, 0x9f, 0x90, 0xea, 0x50, 0x36, 0xd5, 0xc6, 0x3d, 0x4d, 0xbb, 0x30, 0xe4, 0x0d, 0x7b,
        0xe0, // MD5("RHKeyConfig")
        0x03, 0x00, // type = PresetBase + 1
        0x4e, 0x00, 0x4a, 0x00, 0xc8, 0x00, 0xd0, 0x00, 0xcb, 0x00, 0xcd, 0x00, 0x37, 0x00, 0x4f,
        0x00, 0x50, 0x00, 0x51, 0x00, 0x4b, 0x00, 0x4c, 0x00, 0x4d, 0x00, 0x52, 0x00, 0xd1, 0x00,
        0xc9, 0x00, 0x36, 0x00, 0x3a, 0x00, 0x47, 0x00, 0x48, 0x00, 0x49, 0x00, 0x9d, 0x00, 0x1c,
        0x00, 0x39, 0x00, 0x0e, 0x00, 0xb8, 0x00, 0x3b, 0x00, 0x3f, 0x00, 0x3f, 0x00,
    ];

    #[test]
    fn keyset1_preset_matches_shipped_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("keyset1.cfg");
        std::fs::write(&path, KEYSET1_FIXTURE).unwrap();

        let from_file = KeyConfig::load_from_keyset_file(&path).unwrap();
        let hardcoded = KeyConfig::default_preset();

        let mut a = vec![0u16; REAL_KEY_COUNT as usize];
        let mut b = vec![0u16; REAL_KEY_COUNT as usize];
        from_file.get_keys_array(&mut a);
        hardcoded.get_keys_array(&mut b);

        assert_eq!(
            a, b,
            "default_preset must match the decoded shipped keyset1.cfg"
        );
        assert_eq!(from_file.key_type, hardcoded.key_type);
    }

    #[test]
    fn keyset2_preset_matches_shipped_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("keyset2.cfg");
        std::fs::write(&path, KEYSET2_FIXTURE).unwrap();

        let from_file = KeyConfig::load_from_keyset_file(&path).unwrap();
        let hardcoded = KeyConfig::alternate_preset();

        let mut a = vec![0u16; REAL_KEY_COUNT as usize];
        let mut b = vec![0u16; REAL_KEY_COUNT as usize];
        from_file.get_keys_array(&mut a);
        hardcoded.get_keys_array(&mut b);

        assert_eq!(
            a, b,
            "alternate_preset must match the decoded shipped keyset2.cfg"
        );
        assert_eq!(from_file.key_type, hardcoded.key_type);
    }

    #[test]
    fn serde_round_trip() {
        let mut cfg = KeyConfig::default();
        cfg.set_binding("Crouch", 0x2A, 0x36);
        cfg.set_binding("Minimap", 0x32, 0x00);

        let json = serde_json::to_string(&cfg).unwrap();
        let restored: KeyConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.bindings.len(), 2);
        let b = restored.get_binding("Crouch").unwrap();
        assert_eq!(b.primary_key, 0x2A);
        assert_eq!(b.secondary_key, 0x36);
    }
}
