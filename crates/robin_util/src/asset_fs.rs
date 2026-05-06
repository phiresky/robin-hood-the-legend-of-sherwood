//! Unified asset read layer.
//!
//! Two layers on native, one on wasm:
//!
//! 1. **In-memory bundle** (`install_bundle`).  A `BTreeMap<String, Vec<u8>>`
//!    populated at startup from the shipping-datadir's pre-bundled small
//!    assets (`.rhs`, `.rhp`, `.rhm`, `.scb`, `.res`, …).  On wasm this
//!    is the *only* read path — a bundle miss returns `NotFound`.  On
//!    native it's a fast path; misses fall through to disk.
//!
//! 2. **Native filesystem** (`std::fs::read`).  Native only.
//!
//! On `wasm32-unknown-unknown` the engine's synchronous read path is
//! preserved by requiring the bundle to contain every asset the engine
//! will request.  The JS host pre-fetches the bundle bytes and any
//! large per-level paks, then hands all of them to Rust before the
//! game loop starts.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    #[error("asset not found: {0}")]
    NotFound(String),
    #[error("{0}")]
    Io(String),
}

/// Pre-bundled bytes keyed by path.  Paths are normalised (forward
/// slashes, no leading `./`, no leading `Data/` — the shipping converter
/// strips it).
pub type Bundle = std::collections::BTreeMap<String, Vec<u8>>;

static BUNDLE: OnceLock<Arc<Bundle>> = OnceLock::new();
static PRELOADED: OnceLock<Mutex<Bundle>> = OnceLock::new();

/// Install the shipping-datadir in-memory bundle.  Called once at startup
/// by `main_entry` after loading `datadir.bin`.  Returns `Err` if a bundle
/// was already installed.
pub fn install_bundle(bundle: Arc<Bundle>) -> Result<(), Arc<Bundle>> {
    BUNDLE.set(bundle)
}

fn bundle() -> Option<&'static Bundle> {
    BUNDLE.get().map(|a| a.as_ref())
}

fn preloaded() -> &'static Mutex<Bundle> {
    PRELOADED.get_or_init(|| Mutex::new(Bundle::new()))
}

/// Install or replace one asset fetched asynchronously by the host
/// before entering the synchronous game loop.  This is primarily used
/// by wasm builds for large per-level paks kept outside `datadir.bin`.
pub fn install_preloaded_asset<P: AsRef<Path>>(path: P, bytes: Vec<u8>) {
    preloaded()
        .lock()
        .expect("preloaded asset bundle poisoned")
        .insert(bundle_key(path.as_ref()), bytes);
}

/// Normalise a caller path to the key scheme the bundle uses: forward
/// slashes, lowercase, no `./` prefix, no `Data/` prefix.  Case folding
/// matches the native filesystem's `resolve_case_insensitive` behaviour
/// — demo installers ship mixed-case names (`DATA/`, `leicester.rhp`)
/// while the engine asks for them in yet a different case.
pub fn bundle_key(path: &Path) -> String {
    let mut k = path.to_string_lossy().replace('\\', "/");
    while let Some(rest) = k.strip_prefix("./") {
        k = rest.to_string();
    }
    if let Some(rest) = k.strip_prefix("Data/") {
        k = rest.to_string();
    } else if let Some(rest) = k.strip_prefix("data/") {
        k = rest.to_string();
    } else if let Some(rest) = k.strip_prefix("DATA/") {
        k = rest.to_string();
    }
    k.to_ascii_lowercase()
}

/// Read an asset file.  Path is relative to the asset root
/// (`ROBINHOOD_DATA_DIR` on native, the configured base URL on wasm).
pub fn read<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, AssetError> {
    let path = path.as_ref();
    if let Some(b) = bundle()
        && let Some(bytes) = b.get(&bundle_key(path))
    {
        return Ok(bytes.clone());
    }
    {
        let guard = preloaded().lock().expect("preloaded asset bundle poisoned");
        if let Some(bytes) = guard.get(&bundle_key(path)) {
            return Ok(bytes.clone());
        }
    }
    imp::read(path)
}

/// Whether an asset exists.  Checks the bundle first, then falls through
/// to a filesystem stat (native) or synchronous HEAD request (wasm).
pub fn exists<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    if let Some(b) = bundle()
        && b.contains_key(&bundle_key(path))
    {
        return true;
    }
    if preloaded()
        .lock()
        .expect("preloaded asset bundle poisoned")
        .contains_key(&bundle_key(path))
    {
        return true;
    }
    imp::exists(path)
}

/// Resolve an asset-relative path to an absolute `PathBuf` suitable for
/// passing to APIs that require a real path (e.g. SDL's audio loaders on
/// native).  On wasm this returns the path unchanged — the caller must use
/// [`read`] instead of opening by path.
pub fn absolute<P: AsRef<Path>>(path: P) -> PathBuf {
    imp::absolute(path.as_ref())
}

// ---------------------------------------------------------------------------
// Native implementation
// ---------------------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use super::*;

    pub fn read(path: &Path) -> Result<Vec<u8>, AssetError> {
        std::fs::read(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AssetError::NotFound(path.display().to_string())
            } else {
                AssetError::Io(format!("{}: {e}", path.display()))
            }
        })
    }

    pub fn exists(path: &Path) -> bool {
        path.exists()
    }

    pub fn absolute(path: &Path) -> PathBuf {
        path.to_path_buf()
    }
}

// ---------------------------------------------------------------------------
// Wasm implementation: bundle-only.  Anything the engine asks for that
// isn't in the pre-loaded `Bundle` is reported as `NotFound`; the JS
// host is responsible for installing a complete bundle before the
// game loop starts.
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
mod imp {
    use super::*;

    pub fn read(path: &Path) -> Result<Vec<u8>, AssetError> {
        Err(AssetError::NotFound(path.display().to_string()))
    }

    pub fn exists(_path: &Path) -> bool {
        false
    }

    pub fn absolute(path: &Path) -> PathBuf {
        path.to_path_buf()
    }
}
