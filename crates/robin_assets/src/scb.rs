//! `.scb` script-bytecode loader.
//!
//! Each mission ships a compiled script in `Data/Levels/<mission>.scb`.
//! The file holds one or more "classes" (the VM's unit of scoping), each
//! with member variables, functions, and a stream of quads (the VM's
//! 4-tuple instructions).
//!
//! Fully implemented: parses header (magic, version, class count), each
//! class's source filename / class name, member variables with type tags,
//! functions with frame layout, and raw quad streams. Tested against all
//! 39 shipped full-game scripts + the demo script.

use std::fmt;
use std::fs;
use std::path::Path;

// Sim-side data types live in `robin_engine::scb` so the engine can
// consume parsed scripts without depending on robin_assets. The parser
// in this module produces the engine type directly.
pub use robin_engine::scb::{
    ClassEntry, Function, MemberVariable, SCB_MAGIC, SCB_VERSION, ScType, ScbFile, TypeTag,
};
pub use robin_engine::vm::Quad;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    BadMagic { found: [u8; 8] },
    BadVersion { found: f32, expected: f32 },
    Truncated { wanted: usize, available: usize },
    BadUtf8,
    UnknownTypeTag(u8),
    TrailingBytes { left: usize },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::BadMagic { found } => {
                write!(f, "not a .scb file (magic = {:?})", found)
            }
            Error::BadVersion { found, expected } => {
                write!(f, ".scb version {found} != expected {expected}")
            }
            Error::Truncated { wanted, available } => {
                write!(f, "truncated: wanted {wanted} bytes, {available} left")
            }
            Error::BadUtf8 => write!(f, "invalid UTF-8 in string field"),
            Error::UnknownTypeTag(b) => write!(f, "unknown type tag 0x{b:02x}"),
            Error::TrailingBytes { left } => {
                write!(f, "{left} bytes left unparsed after last class")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

/// Parses a `.scb` file: header, all classes, each class's members,
/// functions, and quad stream. Opcode semantics are not decoded — quads
/// are kept as `{u8 op, [u8;8] operands}`.
pub fn parse_bytes(bytes: &[u8]) -> Result<ScbFile, Error> {
    let mut r = Cursor::new(bytes);

    let magic = r.take_array::<8>()?;
    if &magic != SCB_MAGIC {
        return Err(Error::BadMagic { found: magic });
    }

    let version = r.take_f32_le()?;
    if version != SCB_VERSION {
        return Err(Error::BadVersion {
            found: version,
            expected: SCB_VERSION,
        });
    }

    let num_classes = r.take_u32_le()?;
    let mut classes = Vec::with_capacity(num_classes as usize);
    for _ in 0..num_classes {
        classes.push(parse_class(&mut r)?);
    }

    // Every byte in the file should now be accounted for. Anything left
    // over means either my understanding of the format is off or the
    // file has an unknown trailing section.
    let left = r.bytes.len() - r.pos;
    if left != 0 {
        return Err(Error::TrailingBytes { left });
    }

    Ok(ScbFile { version, classes })
}

fn parse_class(r: &mut Cursor<'_>) -> Result<ClassEntry, Error> {
    let source_file = r.take_len_prefixed_string()?;
    let class_name = r.take_len_prefixed_string()?;

    // Member variables: i32 count, i32 total heap size, then N records.
    let mv_count = r.take_i32_le()?;
    let size_of_member_variables = r.take_i32_le()?;
    let mut member_variables = Vec::with_capacity(mv_count.max(0) as usize);
    for _ in 0..mv_count.max(0) {
        member_variables.push(parse_member_variable(r)?);
    }

    // Functions: i32 count, then N records.
    let fn_count = r.take_i32_le()?;
    let mut functions = Vec::with_capacity(fn_count.max(0) as usize);
    for _ in 0..fn_count.max(0) {
        functions.push(parse_function(r)?);
    }

    // Quads: i32 count, then 9 bytes per quad. Operands are opcode-
    // dependent; we keep the raw bytes. On disk the layout is little-
    // endian; BE hosts would need per-opcode byte-swapping.
    let quad_count = r.take_i32_le()?;
    let mut quads = Vec::with_capacity(quad_count.max(0) as usize);
    for _ in 0..quad_count.max(0) {
        let operation = r.take_u8()?;
        let operands = r.take_array::<8>()?;
        quads.push(Quad {
            operation,
            operands,
        });
    }

    Ok(ClassEntry {
        source_file,
        class_name,
        size_of_member_variables,
        member_variables,
        functions,
        quads,
    })
}

fn parse_type(r: &mut Cursor<'_>) -> Result<ScType, Error> {
    let tag_byte = r.take_u8()?;
    let tag = TypeTag::from_u8(tag_byte).ok_or(Error::UnknownTypeTag(tag_byte))?;
    // Native type name is length-prefixed with a single byte (not 4).
    let name_len = r.take_u8()? as usize;
    let name_bytes = r.take(name_len)?;
    let native_type_name = std::str::from_utf8(name_bytes)
        .map(|s| s.to_owned())
        .map_err(|_| Error::BadUtf8)?;
    Ok(ScType {
        tag,
        native_type_name,
    })
}

fn parse_member_variable(r: &mut Cursor<'_>) -> Result<MemberVariable, Error> {
    let ty = parse_type(r)?;
    let name = r.take_len_prefixed_string()?;
    let address = r.take_i32_le()?;
    Ok(MemberVariable { ty, name, address })
}

fn parse_function(r: &mut Cursor<'_>) -> Result<Function, Error> {
    let name = r.take_len_prefixed_string()?;
    let address = r.take_i32_le()?;
    let num_parameters = r.take_i32_le()?;
    let size_of_return_value = r.take_i32_le()?;
    let size_of_parameters = r.take_i32_le()?;
    let size_of_volatile = r.take_i32_le()?;
    let size_of_temporary = r.take_i32_le()?;
    Ok(Function {
        name,
        address,
        num_parameters,
        size_of_return_value,
        size_of_parameters,
        size_of_volatile,
        size_of_temporary,
    })
}

/// Convenience: read+parse a `.scb` file from disk.
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<ScbFile, Error> {
    let p = path.as_ref();
    let resolved =
        robin_engine::sbfile::resolve_case_insensitive(p).unwrap_or_else(|| p.to_path_buf());
    let bytes = fs::read(resolved)?;
    parse_bytes(&bytes)
}

// ----- small LE cursor (kept local — no third-party deps) -----

struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], Error> {
        if self.pos + n > self.bytes.len() {
            return Err(Error::Truncated {
                wanted: n,
                available: self.bytes.len() - self.pos,
            });
        }
        let slice = &self.bytes[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    fn take_array<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        let slice = self.take(N)?;
        let mut arr = [0u8; N];
        arr.copy_from_slice(slice);
        Ok(arr)
    }

    fn take_u8(&mut self) -> Result<u8, Error> {
        Ok(self.take_array::<1>()?[0])
    }

    fn take_u32_le(&mut self) -> Result<u32, Error> {
        Ok(u32::from_le_bytes(self.take_array::<4>()?))
    }

    fn take_i32_le(&mut self) -> Result<i32, Error> {
        Ok(i32::from_le_bytes(self.take_array::<4>()?))
    }

    fn take_f32_le(&mut self) -> Result<f32, Error> {
        Ok(f32::from_le_bytes(self.take_array::<4>()?))
    }

    fn take_len_prefixed_string(&mut self) -> Result<String, Error> {
        let len = self.take_u32_le()? as usize;
        let bytes = self.take(len)?;
        // Scripts were authored on Windows; strings are ASCII/Latin-1 in
        // practice. We accept valid UTF-8 now and can revisit if any
        // shipped .scb holds high-bit bytes.
        std::str::from_utf8(bytes)
            .map(|s| s.to_owned())
            .map_err(|_| Error::BadUtf8)
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    /// Path to the demo mission script, resolved relative to this crate.
    /// Returns None if the datadir isn't checked out (e.g. in CI without
    /// assets).
    fn demo_scb_path() -> Option<std::path::PathBuf> {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let p = std::path::PathBuf::from(manifest_dir)
            .join("../../datadirs/demo/Data/Levels/Dem_Lei_MP.scb");
        p.canonicalize().ok()
    }

    #[test]
    fn rejects_non_scb_magic() {
        let bytes = b"not a scb file, not even close......";
        let err = parse_bytes(bytes).unwrap_err();
        assert!(matches!(err, Error::BadMagic { .. }));
    }

    #[test]
    fn rejects_truncated_header() {
        let bytes = b"SBSCRI";
        let err = parse_bytes(bytes).unwrap_err();
        assert!(matches!(err, Error::Truncated { .. }));
    }

    #[test]
    fn rejects_wrong_version() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(SCB_MAGIC);
        bytes.extend_from_slice(&2.0f32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        let err = parse_bytes(&bytes).unwrap_err();
        match err {
            Error::BadVersion { found, expected } => {
                assert_eq!(expected, 1.5);
                assert_eq!(found, 2.0);
            }
            other => panic!("wrong error: {other:?}"),
        }
    }

    #[test]
    fn empty_class_list_ok() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(SCB_MAGIC);
        bytes.extend_from_slice(&1.5f32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        let scb = parse_bytes(&bytes).unwrap();
        assert_eq!(scb.version, 1.5);
        assert!(scb.classes.is_empty());
    }

    #[test]
    fn rejects_trailing_garbage() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(SCB_MAGIC);
        bytes.extend_from_slice(&1.5f32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&[0xEE; 8]); // unexpected tail
        let err = parse_bytes(&bytes).unwrap_err();
        match err {
            Error::TrailingBytes { left } => assert_eq!(left, 8),
            other => panic!("wrong error: {other:?}"),
        }
    }

    /// Builds a minimal one-class, one-member, one-function, no-quads
    /// .scb in-memory and verifies every field round-trips.
    #[test]
    fn parses_synthetic_single_class() {
        let mut b = Vec::new();
        b.extend_from_slice(SCB_MAGIC);
        b.extend_from_slice(&1.5f32.to_le_bytes());
        b.extend_from_slice(&1u32.to_le_bytes()); // num_classes

        // --- class: source filename + class name
        let src = b"C:\\Temp\\script.scs";
        b.extend_from_slice(&(src.len() as u32).to_le_bytes());
        b.extend_from_slice(src);
        let name = b"StartUp";
        b.extend_from_slice(&(name.len() as u32).to_le_bytes());
        b.extend_from_slice(name);

        // --- member variables: 1 int "iCount" @ address 0, heap size 4
        b.extend_from_slice(&1i32.to_le_bytes()); // mv_count
        b.extend_from_slice(&4i32.to_le_bytes()); // size_of_member_variables
        b.push(TypeTag::Int as u8); // type tag
        b.push(0u8); // native type name length
        let mv_name = b"iCount";
        b.extend_from_slice(&(mv_name.len() as u32).to_le_bytes());
        b.extend_from_slice(mv_name);
        b.extend_from_slice(&0i32.to_le_bytes()); // address

        // --- functions: 1 function "Main" @ address 0
        b.extend_from_slice(&1i32.to_le_bytes()); // fn_count
        let fn_name = b"Main";
        b.extend_from_slice(&(fn_name.len() as u32).to_le_bytes());
        b.extend_from_slice(fn_name);
        b.extend_from_slice(&0i32.to_le_bytes()); // address
        b.extend_from_slice(&0i32.to_le_bytes()); // num_parameters
        b.extend_from_slice(&0i32.to_le_bytes()); // size_of_return_value
        b.extend_from_slice(&0i32.to_le_bytes()); // size_of_parameters
        b.extend_from_slice(&4i32.to_le_bytes()); // size_of_volatile
        b.extend_from_slice(&0i32.to_le_bytes()); // size_of_temporary

        // --- quads: zero of them
        b.extend_from_slice(&0i32.to_le_bytes());

        let scb = parse_bytes(&b).unwrap();
        assert_eq!(scb.classes.len(), 1);
        let c = &scb.classes[0];
        assert_eq!(c.source_file, "C:\\Temp\\script.scs");
        assert_eq!(c.class_name, "StartUp");
        assert_eq!(c.size_of_member_variables, 4);
        assert_eq!(c.member_variables.len(), 1);
        assert_eq!(c.member_variables[0].ty.tag, TypeTag::Int);
        assert_eq!(c.member_variables[0].name, "iCount");
        assert_eq!(c.member_variables[0].address, 0);
        assert_eq!(c.functions.len(), 1);
        assert_eq!(c.functions[0].name, "Main");
        assert_eq!(c.functions[0].size_of_volatile, 4);
        assert!(c.quads.is_empty());
    }

    /// Exercises the parser against the actual shipped demo mission
    /// script. This is the key differential-test hook: if the on-disk
    /// format diverges from my understanding, this fails with an error
    /// pointing at the offset.
    #[test]
    fn parses_shipped_demo_script() {
        let Some(path) = demo_scb_path() else {
            tracing::warn!("skipping: demo .scb not present");
            return;
        };
        let scb = parse_file(&path).expect("demo .scb should parse");
        assert_eq!(scb.version, 1.5);
        assert!(!scb.classes.is_empty());

        // First class is the mission script: "StartUp" authored by
        // Spellbound dev "ECoste" in Windows Temp.
        let first = &scb.classes[0];
        assert_eq!(first.class_name, "StartUp");
        assert!(
            first.source_file.contains("script.scs"),
            "source_file = {:?}",
            first.source_file
        );
        assert!(
            !first.member_variables.is_empty(),
            "StartUp should declare state for iOldSeconds* timers"
        );
        // From the hex dump: seven i32 member variables named iOldSeconds*.
        // Don't over-constrain the count here — the point is non-zero.
        assert!(!first.quads.is_empty(), "StartUp must have bytecode");

        // Sanity-check every class parsed cleanly.
        for c in &scb.classes {
            assert!(!c.class_name.is_empty());
        }
    }

    /// Asserts the demo .scb has non-trivial content across all four
    /// record types. A regression in offset arithmetic typically shows
    /// up as one of these counts going to zero (or the file not fully
    /// consuming).
    #[test]
    fn demo_script_has_content() {
        let Some(path) = demo_scb_path() else { return };
        let scb = parse_file(&path).unwrap();
        let mvars: usize = scb.classes.iter().map(|c| c.member_variables.len()).sum();
        let fns: usize = scb.classes.iter().map(|c| c.functions.len()).sum();
        let quads: usize = scb.classes.iter().map(|c| c.quads.len()).sum();
        assert!(scb.classes.len() > 1, "demo .scb has multiple classes");
        assert!(mvars > 0, "script declares member variables");
        assert!(fns > 0, "script declares functions");
        assert!(quads > 0, "script has bytecode");
    }

    /// Parse every .scb in the full-game datadirs. If the directory
    /// isn't present, the test is silently skipped. Catches format
    /// regressions across all 39 mission scripts.
    #[test]
    fn parses_all_fullgame_scripts() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let levels_dir =
            std::path::PathBuf::from(manifest_dir).join("../../datadirs/fullgame/Data/Levels");
        let Ok(levels_dir) = levels_dir.canonicalize() else {
            tracing::warn!("skipping: fullgame datadirs not present");
            return;
        };

        let mut parsed = 0;
        let mut total_classes = 0;
        let mut total_quads = 0;

        for entry in std::fs::read_dir(&levels_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("scb") {
                continue;
            }
            let scb = parse_file(&path).unwrap_or_else(|e| {
                panic!("{}: {e}", path.display());
            });
            assert!(scb.version == SCB_VERSION);
            for c in &scb.classes {
                assert!(!c.class_name.is_empty());
                total_quads += c.quads.len();
            }
            total_classes += scb.classes.len();
            parsed += 1;
        }

        assert!(parsed > 0, "should have found .scb files");
        tracing::info!("fullgame: {parsed} scripts, {total_classes} classes, {total_quads} quads");
    }
}
