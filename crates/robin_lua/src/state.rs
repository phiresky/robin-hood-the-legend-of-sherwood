//! `MissionLuaState` — the per-mission Lua interpreter wrapper.
//!
//! One instance is created when a mission with a `.lua` file is
//! loaded. It owns:
//!
//! - the `mlua::Lua` state itself (Luau dialect — see crate Cargo
//!   feature note),
//! - registered native bindings (callable from Lua as
//!   `GetActor("Robin")`, `StartSequence()`, etc.),
//! - a custom `require` function rooted at the mission directory
//!   so `require("lib.common")` resolves to the Spellforge `lib/`
//!   folder shipped with the mission.
//!
//! It does not own engine state — engine pointers are passed in per
//! call via [`MissionLuaState::with_host`]. The Lua state lives on
//! the host side (not in `Engine`) because `mlua::Lua` is not
//! serializable or rollback-friendly: see `docs/lua.md` for the
//! single-player-only determinism story.
//!
//! ## Why Luau, not Lua 5.4
//!
//! Luau's [`Lua::sandbox`] freezes the globals table and reroutes
//! script-local writes through a per-script environment, which is
//! the right safety primitive for running arbitrary downloaded
//! missions. Luau also ships without `io`, `os.execute`, or
//! `package.loadlib` — i.e. the destructive corners we'd otherwise
//! have to strip by hand. The trade-off is that Luau has no
//! built-in `require` (the `package` library isn't loaded); we
//! supply our own (see [`install_require`]).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use mlua::Lua;
use robin_engine::natives::GameHost;

use crate::natives::HostPtr;

/// Registry key for the `SequenceCallbacks` table. Hidden from
/// the script's view of `_G` so the sandbox doesn't freeze it.
pub(crate) const SEQUENCE_CALLBACKS_KEY: &str = "robin_lua.sequence_callbacks";

/// Errors produced while loading or driving a mission `.lua`.
#[derive(Debug, thiserror::Error)]
pub enum MissionLuaError {
    #[error("reading {0}: {1}")]
    Io(PathBuf, #[source] std::io::Error),
    #[error("lua error in {0}: {1}")]
    Lua(PathBuf, #[source] mlua::Error),
    #[error("lua error: {0}")]
    Runtime(#[from] mlua::Error),
}

/// One mission's Lua state.
///
/// Created lazily when a mission's `.lua` companion file is present
/// next to its `.rhm`. Dropped when the mission unloads.
pub struct MissionLuaState {
    lua: Lua,
    /// Directory containing the mission's `.lua` file. Used as the
    /// root for the custom `require` resolver.
    mission_dir: PathBuf,
    /// Whether `register_natives` has been called. Guards against
    /// double-registration if a host accidentally rebinds twice.
    natives_registered: bool,
}

impl MissionLuaState {
    /// Create a fresh, empty Lua state for the mission at
    /// `mission_dir`. Natives are not registered yet — the caller
    /// must call [`crate::register_natives`] before loading scripts
    /// (so script top-level code can already reference natives).
    ///
    /// Standard libraries loaded: Luau's default safe set (`string`,
    /// `table`, `math`, `bit32`, `coroutine`, `utf8`, plus the
    /// `buffer` and `vector` Luau-specifics). `io` and `package`
    /// are absent by design — we supply our own scoped `require`.
    pub fn new(mission_dir: impl Into<PathBuf>) -> Result<Self, MissionLuaError> {
        let lua = Lua::new();
        let mission_dir = mission_dir.into();

        // Custom require rooted at the mission dir. Installed
        // before sandbox so it's part of the frozen baseline.
        install_require(&lua, &mission_dir)?;

        // `SequenceCallbacks` is the closure stash for
        // `SequenceCall(fn)` — see `SequenceCall` semantics in
        // `docs/lua.md`. We keep it in the registry rather than
        // `_G` so the sandbox's "globals are frozen" rule doesn't
        // block `SequenceCall` from inserting new ids. Scripts
        // never reach into the table directly (it's a private
        // implementation detail of `SequenceCall`), so hiding it
        // from globals is observation-preserving.
        let sequence_callbacks = lua.create_table()?;
        sequence_callbacks.set("__next_id", 10_000_i32)?;
        lua.set_named_registry_value(SEQUENCE_CALLBACKS_KEY, sequence_callbacks)?;

        // Freeze the global environment. After this call, scripts
        // still read globals normally; writes to `_G` go into a
        // per-script environment table that we can throw away
        // between missions without leaking state.
        //
        // Doing this *after* native registration would freeze them
        // unwritable — fine, since we never want scripts to
        // overwrite engine bindings. Native registration runs in
        // `register_natives` which is called after `new`, so we
        // sandbox there instead. See `register_natives`.

        Ok(Self {
            lua,
            mission_dir,
            natives_registered: false,
        })
    }

    /// Borrow the underlying `mlua::Lua`. Used by the natives module
    /// to register functions onto `globals()`.
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Mission directory the state was initialised with.
    pub fn mission_dir(&self) -> &Path {
        &self.mission_dir
    }

    /// Whether [`crate::register_natives`] has run against this state.
    pub fn natives_registered(&self) -> bool {
        self.natives_registered
    }

    /// Mark natives as registered. Called by [`crate::register_natives`].
    pub(crate) fn mark_natives_registered(&mut self) {
        self.natives_registered = true;
    }

    /// Run `f` with the engine's [`GameHost`] attached as Lua app
    /// data. All registered natives can reach into the host while
    /// `f` is on the stack; once `f` returns, the pointer is
    /// removed so a stray Lua coroutine resumed later can't see
    /// stale state.
    ///
    /// **Safety**: the closure must not stash a reference to the
    /// host that outlives this call (no `lua.create_thread` that
    /// captures host state, no Rust upvalues holding `&mut
    /// GameHost`). All host access happens through registered
    /// native shims, which themselves only run synchronously
    /// inside this scope.
    pub fn with_host<R>(
        &self,
        host: &mut GameHost,
        f: impl FnOnce(&Lua) -> mlua::Result<R>,
    ) -> mlua::Result<R> {
        self.lua.set_app_data(HostPtr::new(host as *mut _));
        let result = f(&self.lua);
        // Always remove, even on Err, so the next call starts
        // clean. `remove_app_data` returns `Option<T>` — discard.
        let _ = self.lua.remove_app_data::<HostPtr>();
        result
    }

    /// Load and execute the mission's `.lua` file. The path is
    /// `<mission_dir>/<stem>.lua`; the leading directory matches
    /// what the custom `require` resolver expects, so `require()`
    /// calls inside the script find their helpers.
    pub fn load_script(&self, stem: &str) -> Result<(), MissionLuaError> {
        let path = self.mission_dir.join(format!("{stem}.lua"));
        let src = std::fs::read(&path).map_err(|e| MissionLuaError::Io(path.clone(), e))?;
        self.lua
            .load(&src)
            .set_name(stem)
            .exec()
            .map_err(|e| MissionLuaError::Lua(path, e))?;
        Ok(())
    }
}

/// Install a `require(path)` global resolving `"foo.bar"` to
/// `<mission_dir>/foo/bar.lua`. Caches each module's return value
/// in a closed-over `HashMap` so repeated requires return the same
/// table — matches Lua's standard semantics.
///
/// Luau doesn't ship `package` / `package.path` / `package.loaded`,
/// so we replicate just the pieces Spellforge mission scripts use.
/// In practice that's `require("lib.common")` and friends — a flat
/// dotted name resolving to a single `.lua` file.
fn install_require(lua: &Lua, mission_dir: &Path) -> Result<(), mlua::Error> {
    let cache: Arc<Mutex<HashMap<String, mlua::Value>>> = Arc::new(Mutex::new(HashMap::new()));
    let root = mission_dir.to_path_buf();

    let require = lua.create_function(move |lua, name: String| {
        if let Some(v) = cache.lock().unwrap().get(&name) {
            return Ok(v.clone());
        }
        let rel = name.replace('.', "/") + ".lua";
        let path = root.join(&rel);
        let src = std::fs::read(&path).map_err(|e| {
            mlua::Error::RuntimeError(format!(
                "require('{name}'): cannot read {}: {e}",
                path.display()
            ))
        })?;
        let chunk = lua.load(&src).set_name(name.clone());
        let value: mlua::Value = chunk
            .eval()
            .map_err(|e| mlua::Error::RuntimeError(format!("require('{name}'): {e}")))?;
        cache.lock().unwrap().insert(name, value.clone());
        Ok(value)
    })?;
    lua.globals().set("require", require)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_state() -> (MissionLuaState, TempDir) {
        let dir = tempfile::tempdir().expect("tempdir");
        let state = MissionLuaState::new(dir.path()).expect("new");
        (state, dir)
    }

    #[test]
    fn require_is_installed() {
        let (state, _dir) = make_state();
        let kind: String = state.lua().load("return type(require)").eval().unwrap();
        assert_eq!(kind, "function");
    }

    /// Confirms our `Lua::new()` baseline doesn't expose `os.execute`
    /// or `io` — Luau doesn't ship them, so the sandbox surface
    /// starts smaller than Lua 5.4. We don't need to nil them
    /// ourselves.
    #[test]
    fn dangerous_libs_absent_by_default() {
        let (state, _dir) = make_state();
        let io_kind: String = state.lua().load("return type(io)").eval().unwrap();
        assert_eq!(io_kind, "nil", "Luau must not load `io`");
        // `os` is present in Luau but only with a minimal set
        // (`os.time`, `os.clock`, `os.date`, `os.difftime`). The
        // dangerous entries (`execute`, `remove`, `getenv`, …)
        // aren't there. Spot-check `os.execute`.
        let exec: String = state
            .lua()
            .load("return type((os or {}).execute)")
            .eval()
            .unwrap();
        assert_eq!(exec, "nil");
    }

    #[test]
    fn load_script_runs_top_level() {
        let (state, dir) = make_state();
        fs::write(dir.path().join("mission.lua"), "_G.mission_loaded = 42\n").unwrap();
        state.load_script("mission").unwrap();
        let v: i64 = state.lua().globals().get("mission_loaded").unwrap();
        assert_eq!(v, 42);
    }

    #[test]
    fn require_resolves_lib_subdir() {
        let (state, dir) = make_state();
        fs::create_dir(dir.path().join("lib")).unwrap();
        fs::write(
            dir.path().join("lib/common.lua"),
            "return { hello = 'world' }\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("mission.lua"),
            "_G.greeting = require('lib.common').hello\n",
        )
        .unwrap();
        state.load_script("mission").unwrap();
        let g: String = state.lua().globals().get("greeting").unwrap();
        assert_eq!(g, "world");
    }

    /// Calling `require` twice on the same module returns the same
    /// table — matches stock Lua's `package.loaded` caching, which
    /// Spellforge's `lib/common.lua` relies on (it stashes
    /// mission-scoped state on the returned table).
    #[test]
    fn require_caches_modules() {
        let (state, dir) = make_state();
        fs::write(dir.path().join("m.lua"), "return {}\n").unwrap();
        let same: bool = state
            .lua()
            .load("return require('m') == require('m')")
            .eval()
            .unwrap();
        assert!(same, "require must cache");
    }
}
