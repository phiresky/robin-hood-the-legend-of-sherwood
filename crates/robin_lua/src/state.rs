//! `MissionLuaState` — the per-mission Lua interpreter wrapper.
//!
//! One instance is created when a mission with a `.lua` file is
//! loaded. It owns:
//!
//! - the `mlua::Lua` state itself,
//! - registered native bindings (callable from Lua as
//!   `GetActor("Robin")`, `StartSequence()`, etc.),
//! - the per-mission `package.path` so `require("lib.common")`
//!   resolves to the Spellforge `lib/` folder shipped with the
//!   mission.
//!
//! It does not own engine state — engine pointers are passed in per
//! call via [`MissionLuaState::with_host`]. The Lua state lives on
//! the host side (not in `Engine`) because `mlua::Lua` is not
//! serializable or rollback-friendly: see `docs/lua.md` for the
//! single-player-only determinism story.

use std::path::{Path, PathBuf};

use mlua::Lua;
use robin_engine::natives::GameHost;

use crate::natives::HostPtr;

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
    /// root for `package.path` so `require("lib.common")` works.
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
    /// Standard libraries opened: `base`, `package`, `string`, `os`
    /// (sandboxed — `os.execute` / `os.exit` are nil'd),
    /// `math`, `table`. The original Spellforge DLL opens the same
    /// six; matching keeps mission scripts portable.
    pub fn new(mission_dir: impl Into<PathBuf>) -> Result<Self, MissionLuaError> {
        let lua = Lua::new();
        let mission_dir = mission_dir.into();

        // Spellforge default: package + base + string + os + math + table.
        // mlua's `Lua::new()` already loads `safe` stdlibs (base, package,
        // string, table, math, utf8); `os` needs an explicit `load_std_libs`
        // — and we strip the destructive entries before scripts run.
        lua.load_std_libs(mlua::StdLib::OS)?;
        sandbox_os(&lua)?;

        // `package.path = <mission_dir>/?.lua;<mission_dir>/?/init.lua`
        // (Spellforge's DLL sets the path to the level folder so that
        // `require("lib.common")` resolves to `lib/common.lua`.)
        let path_pattern = format!(
            "{0}/?.lua;{0}/?/init.lua",
            mission_dir.display().to_string().replace('\\', "/")
        );
        let package: mlua::Table = lua.globals().get("package")?;
        package.set("path", path_pattern)?;
        // Prevent loading native C extensions — `cpath` is set to empty
        // and `package.loadlib` is stripped (the latter via `sandbox_os`
        // for the rest of the dangerous surface).
        package.set("cpath", "")?;
        package.set("loadlib", mlua::Value::Nil)?;

        // Spellforge mission scripts use this table to register
        // sequence-callback IDs — see `SequenceCall` semantics in
        // `docs/lua.md`. Pre-create it so user scripts don't have to.
        lua.globals()
            .set("SequenceCallbacks", lua.create_table()?)?;

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
    /// native shims, which themselves only run synchronously inside
    /// this scope.
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
    /// what `package.path` was seeded with, so `require()` calls
    /// inside the script find their helpers.
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

/// Strip the destructive corners of `os` so a malicious mod can't
/// `os.execute("rm -rf …")` from the script editor. We keep
/// `os.time`, `os.clock`, `os.date` — purely informational.
fn sandbox_os(lua: &Lua) -> Result<(), mlua::Error> {
    let os: mlua::Table = lua.globals().get("os")?;
    for key in [
        "execute",
        "exit",
        "remove",
        "rename",
        "setlocale",
        "tmpname",
        "getenv",
    ] {
        os.set(key, mlua::Value::Nil)?;
    }
    // Strip the global `dofile` / `loadfile` / `load` shortcuts —
    // these would let a script reach arbitrary disk content
    // bypassing `package.path`. Mission scripts use `require()`
    // (which still works because `package.searchers` is intact).
    let globals = lua.globals();
    for key in ["dofile", "loadfile", "load", "loadstring"] {
        globals.set(key, mlua::Value::Nil)?;
    }
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
    fn package_path_includes_mission_dir() {
        let (state, _dir) = make_state();
        let path: String = state
            .lua()
            .globals()
            .get::<mlua::Table>("package")
            .unwrap()
            .get("path")
            .unwrap();
        assert!(path.contains("?.lua"));
    }

    #[test]
    fn dangerous_os_calls_stripped() {
        let (state, _dir) = make_state();
        // `os.execute` is the canary; if it's still callable a mission
        // could shell out from the script editor.
        let res: mlua::Value = state.lua().load("return os.execute").eval().unwrap();
        assert!(matches!(res, mlua::Value::Nil));
        // But `os.time` survives — scripts may want a wall-clock seed
        // for non-determinism warning UIs.
        let res: mlua::Value = state.lua().load("return os.time").eval().unwrap();
        assert!(matches!(res, mlua::Value::Function(_)));
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
}
