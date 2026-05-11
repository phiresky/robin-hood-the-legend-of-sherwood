//! Diagnostic: probe every `.lua` file inside `datadirs/mods/` (and
//! the shared `lib_*.zip`) for Luau parseability.
//!
//! Run from the repo root:
//!
//! ```text
//! cargo run --example check_mod_lua -p robin_lua
//! ```
//!
//! For each `.lua` entry it tries `Lua::load(src).into_function()`,
//! which compiles the source but doesn't execute it — so missing
//! native globals (`Initialize`, `GetActor`, …) don't cause false
//! failures. The only thing we're checking is whether Luau's parser
//! accepts the syntax.
//!
//! Spellforge missions were authored against LuaJIT 2.1. The Luau
//! parser rejects a few LuaJIT-isms (`bit.*` module names, `goto`
//! syntax differences, the `unpack` global, …) so this check
//! surfaces anything we'd need to handle in a compat shim before
//! the mission can actually run.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use mlua::Lua;

fn main() -> ExitCode {
    let mods_root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("datadirs/mods"));

    if !mods_root.is_dir() {
        eprintln!("mods root not found: {}", mods_root.display());
        return ExitCode::FAILURE;
    }

    let lua = Lua::new();
    let mut results: Vec<Result_> = Vec::new();

    // Walk every zip in <mods_root>/**/<*.zip> and probe each
    // .lua entry it contains. Mod zips and the shared lib zip
    // both follow the same shape — just the `.lua` paths inside
    // differ.
    for zip_path in find_zips(&mods_root) {
        match scan_zip(&lua, &zip_path) {
            Ok(mut rows) => results.append(&mut rows),
            Err(e) => {
                results.push(Result_ {
                    zip: zip_path.clone(),
                    entry: "<zip open>".into(),
                    bytes: 0,
                    outcome: Outcome::ZipError(e),
                });
            }
        }
    }

    // Sort: errors first, grouped by zip.
    results.sort_by(|a, b| {
        let aerr = !matches!(a.outcome, Outcome::Ok);
        let berr = !matches!(b.outcome, Outcome::Ok);
        berr.cmp(&aerr).then(a.zip.cmp(&b.zip)).then(a.entry.cmp(&b.entry))
    });

    let mut ok = 0usize;
    let mut failed = 0usize;
    for r in &results {
        match &r.outcome {
            Outcome::Ok => {
                ok += 1;
                println!("OK    {:>7} bytes  {}!{}", r.bytes, short(&r.zip, &mods_root), r.entry);
            }
            Outcome::ParseError(msg) => {
                failed += 1;
                println!(
                    "FAIL  {:>7} bytes  {}!{}\n      {}",
                    r.bytes,
                    short(&r.zip, &mods_root),
                    r.entry,
                    msg.lines().next().unwrap_or(msg)
                );
            }
            Outcome::ZipError(msg) => {
                failed += 1;
                println!("ZIP   {}: {msg}", short(&r.zip, &mods_root));
            }
        }
    }
    println!("\n{ok} parsed, {failed} failed");
    if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

enum Outcome {
    Ok,
    ParseError(String),
    ZipError(String),
}

struct Result_ {
    zip: PathBuf,
    entry: String,
    bytes: usize,
    outcome: Outcome,
}

fn find_zips(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk(root, &mut out);
    out.sort();
    out
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() {
            walk(&p, out);
        } else if p
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("zip"))
        {
            out.push(p);
        }
    }
}

fn scan_zip(lua: &Lua, zip_path: &Path) -> Result<Vec<Result_>, String> {
    let file = fs::File::open(zip_path).map_err(|e| format!("open: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("not a zip: {e}"))?;
    let mut rows = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| format!("entry {i}: {e}"))?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_owned();
        if !name.to_ascii_lowercase().ends_with(".lua") {
            continue;
        }
        let mut bytes = Vec::with_capacity(entry.size() as usize);
        entry
            .read_to_end(&mut bytes)
            .map_err(|e| format!("read {name}: {e}"))?;
        let outcome = match lua.load(&bytes).set_name(&name).into_function() {
            Ok(_) => Outcome::Ok,
            Err(e) => Outcome::ParseError(e.to_string()),
        };
        rows.push(Result_ {
            zip: zip_path.to_owned(),
            entry: name,
            bytes: bytes.len(),
            outcome,
        });
    }
    Ok(rows)
}

fn short(zip: &Path, root: &Path) -> String {
    zip.strip_prefix(root)
        .unwrap_or(zip)
        .display()
        .to_string()
}
