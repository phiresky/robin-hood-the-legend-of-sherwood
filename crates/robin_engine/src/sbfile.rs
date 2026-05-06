//! Read-only filesystem abstraction for loading game data files.
//!
//! All Rust-side persistence uses serde (JSON). SbFile only reads
//! binary game data (`.cpf` profiles, level files, sprite data, etc.).

use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[cfg(not(target_arch = "wasm32"))]
use std::fs;

pub const SBFILE_NO_ERROR: i32 = 0;
pub const SBFILE_ERROR_FILE_NOT_FOUND: i32 = -1;
pub const SBFILE_ERROR_NO_FILE: i32 = -4;
pub const SBFILE_ERROR_READ: i32 = -5;
pub const SBFILE_ERROR_SEEK: i32 = -7;
pub const SBFILE_ERROR_PATH_ALREADY_PRESENT: i32 = -10;
pub const SBFILE_ERROR_PATH_NOT_IN_SET: i32 = -11;

pub const SB_FILE_READ: i32 = 0x01;

static ALTERNATE_PATHS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static OVERLAY_PATHS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static PRIMARY_PATH: Mutex<Option<String>> = Mutex::new(None);

pub struct SbFile {
    /// Game data is short and read sequentially / seekably, so we always
    /// slurp the whole file into memory and drive it with a `Cursor`.
    /// That lets `SbFile::open` uniformly consume bytes from the native
    /// filesystem *or* the shipping-datadir byte store hosted in
    /// `robin_util::asset_fs` without a type split.
    file: Cursor<Vec<u8>>,
    size: u64,
    position: u64,
    last_error: i32,
    version: u32,
}

#[cfg(target_arch = "wasm32")]
pub fn resolve_case_insensitive(path: &Path) -> Option<PathBuf> {
    let path_str = path.to_str()?;
    let normalised = path_str.replace('\\', "/");
    let path = Path::new(&normalised);
    // No `read_dir` on wasm, so we can't walk for case variants.
    // Shipping datadirs authored for wasm use exact-cased paths; a
    // single `asset_fs::exists` probe is enough.
    if robin_util::asset_fs::exists(path) {
        Some(path.to_path_buf())
    } else {
        None
    }
}

#[cfg(not(target_arch = "wasm32"))]
// Walks every component case-insensitively. Shipping datadirs use mixed
// casing across components (`DATA/` uppercase, `data/` lowercase), so
// case-folding has to apply to every component, not just the leaf.
// Dotfile entries (names starting with `.`) are skipped during the
// case-fold scan.
pub fn resolve_case_insensitive(path: &Path) -> Option<PathBuf> {
    let path_str = path.to_str()?;
    let normalised = path_str.replace('\\', "/");
    let path = Path::new(&normalised);
    let mut components = path.components().peekable();
    let mut resolved = match components.peek() {
        Some(std::path::Component::RootDir) => {
            components.next();
            PathBuf::from("/")
        }
        _ => PathBuf::from("."),
    };
    for component in components {
        let target = component.as_os_str().to_str()?;
        let candidate = resolved.join(target);
        if candidate.exists() {
            resolved = candidate;
            continue;
        }
        let target_lower = target.to_ascii_lowercase();
        let mut found = false;
        if let Ok(entries) = fs::read_dir(&resolved) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str()
                    && !name.starts_with('.')
                    && name.to_ascii_lowercase() == target_lower
                {
                    resolved = entry.path();
                    found = true;
                    break;
                }
            }
        }
        if !found {
            return None;
        }
    }
    Some(resolved)
}

/// Resolve a game-data path to an actual filesystem path.
///
/// Tries the path directly (case-insensitive), then each registered alternate
/// path.  Returns `None` if the file cannot be found anywhere.  Used by the
/// video player to obtain a real path for ffmpeg to open.
pub fn resolve_data_path(path: &str) -> Option<PathBuf> {
    let normalised = path.replace('\\', "/");
    let p = Path::new(&normalised);

    // Overlay paths intentionally take precedence over the primary datadir.
    let overlay_paths = OVERLAY_PATHS.lock().unwrap();
    for overlay in overlay_paths.iter() {
        let full = format!("{}/{}", overlay, normalised);
        if let Some(resolved) = resolve_case_insensitive(Path::new(&full))
            && resolved.is_file()
        {
            return Some(resolved);
        }
    }
    drop(overlay_paths);

    if let Some(primary) = PRIMARY_PATH.lock().unwrap().clone() {
        let full = format!("{}/{}", primary, normalised);
        if let Some(resolved) = resolve_case_insensitive(Path::new(&full))
            && resolved.is_file()
        {
            return Some(resolved);
        }
    }

    // Direct path
    if let Some(resolved) = resolve_case_insensitive(p)
        && resolved.is_file()
    {
        return Some(resolved);
    }

    // Alternate paths
    let alt_paths = ALTERNATE_PATHS.lock().unwrap();
    for alt in alt_paths.iter() {
        if let Some(primary) = PRIMARY_PATH.lock().unwrap().clone() {
            let full = format!("{}/{}/{}", primary, alt, normalised);
            if let Some(resolved) = resolve_case_insensitive(Path::new(&full))
                && resolved.is_file()
            {
                return Some(resolved);
            }
        }
        let full = format!("{}/{}", alt, normalised);
        if let Some(resolved) = resolve_case_insensitive(Path::new(&full))
            && resolved.is_file()
        {
            return Some(resolved);
        }
    }

    None
}

/// Read `path` as bytes, honouring case-insensitive resolution on native
/// for datadirs that use mixed case on the wire (e.g. demo installers
/// ship `DATA/` uppercase).
///
/// Per-path NotFound logs at `trace` (expected fallthrough during
/// alternate-path search); any *other* error (network failure, HTTP
/// 5xx, permission denied) is a real problem and logs at `warn` —
/// silently swallowing those turned a wasm network blip into "file
/// missing" and cost us an afternoon of debugging.
///
/// Note: the original release also fired a file-not-found callback on
/// miss to drive an "insert CD" disc-swap prompt. The Rust port ships
/// from a flat datadir, has no CD-media support, and therefore has no
/// equivalent — intentionally dropped.
fn try_read(path: &str) -> Option<Vec<u8>> {
    match robin_util::asset_fs::read(path) {
        Ok(bytes) => return Some(bytes),
        Err(robin_util::asset_fs::AssetError::NotFound(_)) => {
            tracing::trace!("asset {path}: not found");
        }
        Err(e) => tracing::warn!("asset read failed for {path}: {e}"),
    }
    if let Some(resolved) = resolve_case_insensitive(Path::new(path)) {
        match robin_util::asset_fs::read(&resolved) {
            Ok(bytes) => return Some(bytes),
            Err(robin_util::asset_fs::AssetError::NotFound(_)) => {
                tracing::trace!(
                    "asset {} (case-resolved from {path}): not found",
                    resolved.display()
                );
            }
            Err(e) => tracing::warn!(
                "asset read failed for {} (case-resolved from {path}): {e}",
                resolved.display()
            ),
        }
    }
    None
}

impl SbFile {
    pub fn open(path: &str, _flags: i32) -> Result<Self, i32> {
        let normalised = path.replace('\\', "/");
        let overlay_paths = OVERLAY_PATHS.lock().unwrap();
        for overlay in overlay_paths.iter() {
            if let Some(bytes) = try_read(&format!("{overlay}/{normalised}")) {
                return Ok(Self::from_bytes(bytes));
            }
        }
        drop(overlay_paths);
        if let Some(primary) = PRIMARY_PATH.lock().unwrap().clone()
            && let Some(bytes) = try_read(&format!("{primary}/{normalised}"))
        {
            return Ok(Self::from_bytes(bytes));
        }
        if let Some(bytes) = try_read(&normalised) {
            return Ok(Self::from_bytes(bytes));
        }
        let alt_paths = ALTERNATE_PATHS.lock().unwrap();
        for alt in alt_paths.iter() {
            if let Some(primary) = PRIMARY_PATH.lock().unwrap().clone()
                && let Some(bytes) = try_read(&format!("{primary}/{alt}/{normalised}"))
            {
                return Ok(Self::from_bytes(bytes));
            }
            if let Some(bytes) = try_read(&format!("{alt}/{normalised}")) {
                return Ok(Self::from_bytes(bytes));
            }
        }
        tracing::warn!(
            "SbFile::open: {normalised} not found (tried direct + {} alternate paths)",
            alt_paths.len()
        );
        Err(SBFILE_ERROR_FILE_NOT_FOUND)
    }

    fn from_bytes(bytes: Vec<u8>) -> Self {
        let size = bytes.len() as u64;
        SbFile {
            file: Cursor::new(bytes),
            size,
            position: 0,
            last_error: SBFILE_NO_ERROR,
            version: 0,
        }
    }

    pub fn read_all(path: &str) -> Result<Vec<u8>, i32> {
        let mut file = Self::open(path, SB_FILE_READ)?;
        let mut bytes = vec![0; file.get_size() as usize];
        file.serialize_bytes(&mut bytes)?;
        Ok(bytes)
    }

    pub fn read(&mut self, buf: &mut [u8]) -> i32 {
        match self.file.read_exact(buf) {
            Ok(()) => {
                self.position += buf.len() as u64;
                self.last_error = SBFILE_NO_ERROR;
                SBFILE_NO_ERROR
            }
            Err(_) => {
                self.last_error = SBFILE_ERROR_READ;
                SBFILE_ERROR_READ
            }
        }
    }

    pub fn skip(&mut self, distance: i64, mode: u32) -> i32 {
        let seek_from = match mode {
            0 => SeekFrom::Start(distance as u64),
            1 => SeekFrom::Current(distance),
            2 => SeekFrom::End(distance),
            other => {
                tracing::warn!("SbFile::skip: unknown mode {other}, falling back to SEEK_CUR");
                SeekFrom::Current(distance)
            }
        };
        match self.file.seek(seek_from) {
            Ok(pos) => {
                self.position = pos;
                self.last_error = SBFILE_NO_ERROR;
                SBFILE_NO_ERROR
            }
            Err(_) => {
                self.last_error = SBFILE_ERROR_SEEK;
                SBFILE_ERROR_SEEK
            }
        }
    }

    pub fn tell(&mut self) -> u64 {
        self.file.stream_position().unwrap_or(self.position)
    }
    pub fn get_size(&self) -> u64 {
        self.size
    }
    /// True once the cursor has reached the end of the in-memory buffer.
    pub fn is_eof(&self) -> bool {
        self.position >= self.size
    }
    pub fn is_read_mode(&self) -> bool {
        true
    }
    pub fn is_write_mode(&self) -> bool {
        false
    }
    pub fn get_version(&self) -> u32 {
        self.version
    }
    pub fn set_version(&mut self, v: u32) {
        self.version = v;
    }

    // ── Binary readers ───────────────────────────────────────────

    pub fn serialize_bytes(&mut self, buf: &mut [u8]) -> Result<(), i32> {
        if self.read(buf) < 0 {
            Err(self.last_error)
        } else {
            Ok(())
        }
    }
    pub fn serialize_u8(&mut self, val: &mut u8) -> Result<(), i32> {
        let mut b = [0u8; 1];
        self.serialize_bytes(&mut b)?;
        *val = b[0];
        Ok(())
    }
    pub fn serialize_i8(&mut self, val: &mut i8) -> Result<(), i32> {
        let mut b = 0u8;
        self.serialize_u8(&mut b)?;
        *val = b as i8;
        Ok(())
    }
    pub fn serialize_u16(&mut self, val: &mut u16) -> Result<(), i32> {
        let mut b = [0u8; 2];
        self.serialize_bytes(&mut b)?;
        *val = u16::from_le_bytes(b);
        Ok(())
    }
    pub fn serialize_i16(&mut self, val: &mut i16) -> Result<(), i32> {
        let mut b = [0u8; 2];
        self.serialize_bytes(&mut b)?;
        *val = i16::from_le_bytes(b);
        Ok(())
    }
    pub fn serialize_u32(&mut self, val: &mut u32) -> Result<(), i32> {
        let mut b = [0u8; 4];
        self.serialize_bytes(&mut b)?;
        *val = u32::from_le_bytes(b);
        Ok(())
    }
    pub fn serialize_i32(&mut self, val: &mut i32) -> Result<(), i32> {
        let mut b = [0u8; 4];
        self.serialize_bytes(&mut b)?;
        *val = i32::from_le_bytes(b);
        Ok(())
    }
    pub fn serialize_u64(&mut self, val: &mut u64) -> Result<(), i32> {
        let mut b = [0u8; 8];
        self.serialize_bytes(&mut b)?;
        *val = u64::from_le_bytes(b);
        Ok(())
    }
    pub fn serialize_i64(&mut self, val: &mut i64) -> Result<(), i32> {
        let mut b = [0u8; 8];
        self.serialize_bytes(&mut b)?;
        *val = i64::from_le_bytes(b);
        Ok(())
    }
    pub fn serialize_f32(&mut self, val: &mut f32) -> Result<(), i32> {
        let mut b = [0u8; 4];
        self.serialize_bytes(&mut b)?;
        *val = f32::from_le_bytes(b);
        Ok(())
    }
    pub fn serialize_bool(&mut self, val: &mut bool) -> Result<(), i32> {
        let mut b = 0u8;
        self.serialize_u8(&mut b)?;
        *val = b != 0;
        Ok(())
    }
    pub fn serialize_version(&mut self) -> Result<(), i32> {
        let mut v = 0u32;
        self.serialize_u32(&mut v)?;
        self.version = v;
        Ok(())
    }
    pub fn serialize_string(&mut self, s: &mut String) -> Result<(), i32> {
        let mut len = 0u16;
        self.serialize_u16(&mut len)?;
        let mut bytes = vec![0u8; len as usize];
        self.serialize_bytes(&mut bytes)?;
        *s = String::from_utf8_lossy(&bytes).into_owned();
        Ok(())
    }
    pub fn skip_padding(&mut self, n: usize) -> Result<(), i32> {
        let mut buf = vec![0u8; n];
        self.serialize_bytes(&mut buf)
    }
    pub fn validate_stream(&mut self, fingerprint: &str) -> Result<(), i32> {
        use crate::md5::Md5Ctx;
        let mut ctx = Md5Ctx::new();
        ctx.update(fingerprint.as_bytes());
        ctx.finalize();
        let expected = ctx.raw_digest_bytes();
        let mut buf = [0u8; 16];
        self.serialize_bytes(&mut buf)?;
        if buf != expected {
            // The original raised a fatal error here; we surface this as an
            // error-level log + propagated `Err` rather than aborting the
            // process so callers can fail the asset load instead of crashing.
            tracing::error!(
                "ValidateStream: digital signature mismatch for '{}'",
                fingerprint
            );
            return Err(SBFILE_ERROR_READ);
        }
        Ok(())
    }
    pub fn checkpoint(&mut self) -> Result<(), i32> {
        let mut m = 0u16;
        self.serialize_u16(&mut m)?;
        if m != 0x7777 {
            tracing::warn!("CHECKPOINT: shifted (0x{:04x})", m);
            return Err(SBFILE_ERROR_READ);
        }
        Ok(())
    }

    /// Clears `line`, then loops reading one byte at a time, appending
    /// it unless the byte is `\n` or `\r`, until a `\n` is consumed or
    /// EOF is reached. Returns `!self.is_eof()` — i.e. true if the file
    /// may still have more data, false if EOF was reached (whether
    /// mid-line or just past the terminating newline).
    pub fn read_line(&mut self, line: &mut String) -> bool {
        line.clear();
        let mut current = 0u8;
        while current != b'\n' && !self.is_eof() {
            let mut byte = [0u8; 1];
            match self.file.read(&mut byte) {
                Ok(0) => break,
                Ok(_) => {
                    self.position += 1;
                    current = byte[0];
                    if current != 0x0A && current != 0x0D {
                        line.push(current as char);
                    }
                }
                Err(_) => {
                    self.last_error = SBFILE_ERROR_READ;
                    return false;
                }
            }
        }
        !self.is_eof()
    }

    pub fn exists(path: &str) -> bool {
        let n = path.replace('\\', "/");
        let overlays = OVERLAY_PATHS.lock().unwrap();
        for overlay in overlays.iter() {
            let c = format!("{}/{}", overlay, n);
            if Path::new(&c).exists() || resolve_case_insensitive(Path::new(&c)).is_some() {
                return true;
            }
        }
        drop(overlays);
        if let Some(primary) = PRIMARY_PATH.lock().unwrap().clone() {
            let c = format!("{}/{}", primary, n);
            if Path::new(&c).exists() || resolve_case_insensitive(Path::new(&c)).is_some() {
                return true;
            }
        }
        if robin_util::asset_fs::exists(&n) {
            return true;
        }
        let p = Path::new(&n);
        if p.exists() {
            return true;
        }
        if resolve_case_insensitive(p).is_some() {
            return true;
        }
        let alts = ALTERNATE_PATHS.lock().unwrap();
        for alt in alts.iter() {
            if let Some(primary) = PRIMARY_PATH.lock().unwrap().clone() {
                let c = format!("{}/{}/{}", primary, alt, n);
                if Path::new(&c).exists() || resolve_case_insensitive(Path::new(&c)).is_some() {
                    return true;
                }
            }
            let c = format!("{}/{}", alt, n);
            if Path::new(&c).exists() || resolve_case_insensitive(Path::new(&c)).is_some() {
                return true;
            }
        }
        false
    }

    pub fn add_alternate_path(path: &str) -> i32 {
        let mut p = ALTERNATE_PATHS.lock().unwrap();
        if p.iter().any(|x| x == path) {
            return SBFILE_ERROR_PATH_ALREADY_PRESENT;
        }
        p.push(path.to_string());
        SBFILE_NO_ERROR
    }

    pub fn add_overlay_path(path: &str) -> i32 {
        let mut p = OVERLAY_PATHS.lock().unwrap();
        if p.iter().any(|x| x == path) {
            return SBFILE_ERROR_PATH_ALREADY_PRESENT;
        }
        p.push(path.to_string());
        SBFILE_NO_ERROR
    }

    pub fn overlay_paths() -> Vec<String> {
        OVERLAY_PATHS.lock().unwrap().clone()
    }

    pub fn set_primary_path(path: &str) -> i32 {
        *PRIMARY_PATH.lock().unwrap() = Some(path.to_string());
        SBFILE_NO_ERROR
    }

    pub fn remove_alternate_path(path: &str) -> i32 {
        let mut p = ALTERNATE_PATHS.lock().unwrap();
        if let Some(i) = p.iter().position(|x| x == path) {
            p.remove(i);
            SBFILE_NO_ERROR
        } else {
            SBFILE_ERROR_PATH_NOT_IN_SET
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_and_read() {
        let dir = std::env::temp_dir().join("sbfile_ro_test");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test.bin");
        fs::write(&path, b"Hello").unwrap();
        let mut f = SbFile::open(path.to_str().unwrap(), SB_FILE_READ).unwrap();
        let mut buf = [0u8; 5];
        assert_eq!(f.read(&mut buf), SBFILE_NO_ERROR);
        assert_eq!(&buf, b"Hello");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn deserialize_u32_le() {
        let dir = std::env::temp_dir().join("sbfile_ro_u32");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("u32.bin");
        fs::write(&path, [0xEF, 0xBE, 0xAD, 0xDE]).unwrap();
        let mut f = SbFile::open(path.to_str().unwrap(), SB_FILE_READ).unwrap();
        let mut v = 0u32;
        f.serialize_u32(&mut v).unwrap();
        assert_eq!(v, 0xDEADBEEF);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn deserialize_string() {
        let dir = std::env::temp_dir().join("sbfile_ro_str");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("str.bin");
        fs::write(&path, [0x05, 0x00, b'h', b'e', b'l', b'l', b'o']).unwrap();
        let mut f = SbFile::open(path.to_str().unwrap(), SB_FILE_READ).unwrap();
        let mut s = String::new();
        f.serialize_string(&mut s).unwrap();
        assert_eq!(s, "hello");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn checkpoint_valid() {
        let dir = std::env::temp_dir().join("sbfile_ro_chk");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("chk.bin");
        fs::write(&path, [0x77, 0x77]).unwrap();
        let mut f = SbFile::open(path.to_str().unwrap(), SB_FILE_READ).unwrap();
        f.checkpoint().unwrap();
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn alternate_paths() {
        let dir = std::env::temp_dir().join("sbfile_ro_alt");
        let _ = fs::create_dir_all(&dir);
        fs::write(dir.join("secret.dat"), b"x").unwrap();
        assert!(!SbFile::exists("secret.dat"));
        assert_eq!(
            SbFile::add_alternate_path(dir.to_str().unwrap()),
            SBFILE_NO_ERROR
        );
        assert!(SbFile::exists("secret.dat"));
        assert_eq!(
            SbFile::remove_alternate_path(dir.to_str().unwrap()),
            SBFILE_NO_ERROR
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
