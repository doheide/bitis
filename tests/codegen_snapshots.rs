//! Snapshot tests of the generated code.
//!
//! For every `tests/snapshots/<name>.bitis` the generated Rust, C++, Python and Dart files are compared with
//! the committed files in `tests/snapshots/<name>/`. The compiler version is replaced by `{BITIS_VERSION}`
//! in the snapshots. To update the snapshots after an intended change run
//! `BITIS_UPDATE_SNAPSHOTS=1 cargo test --test codegen_snapshots` and review the diff.

use std::fs;
use std::path::{Path, PathBuf};

use bitis_lib::*;

fn snapshot_dir() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("snapshots") }

fn compile_file(path: &Path) -> BitisProcessed {
    let files = vec![SourceFile { name: path.file_name().unwrap().to_string_lossy().to_string(),
        content: fs::read_to_string(path).unwrap() }];
    match compile_sources(&files) {
        Ok(p) => p,
        Err(errors) => panic!("{}", errors.iter().map(|e| e.render(&files)).collect::<Vec<_>>().join("\n")),
    }
}

fn normalize(s: &str) -> String {
    s.replace(&format!("\"{}\"", BITIS_COMPILER_VERSION), "\"{BITIS_VERSION}\"")
        .replace(&format!("'{}'", BITIS_COMPILER_VERSION), "'{BITIS_VERSION}'")
        .replace(&format!("bitis compiler {} (", BITIS_COMPILER_VERSION), "bitis compiler {BITIS_VERSION} (")
}

fn generate(name: &str, p: &BitisProcessed) -> Vec<(String, String)> {
    let py = render_python(p, name).unwrap();
    vec![
        (format!("{name}.rs"), render_rust(p).unwrap()),
        (format!("{name}.h"), render_cpp(p, "bitis_lib.h", &stamp_name_from(name)).unwrap()),
        ("py_messages.rs".to_string(), py.messages_rs),
        ("py_pyrust.rs".to_string(), py.pyrust_rs),
        ("py_lib.rs".to_string(), py.lib_rs),
        (format!("{name}.pyi"), py.pyi),
        (format!("{name}.dart"), render_dart(p).unwrap()),
        ("canonical_schema.txt".to_string(), format!("{}{}\n", canonical_schema(p), schema_hash_hex(p))),
    ]
}

fn check_snapshot(name: &str) {
    let p = compile_file(&snapshot_dir().join(format!("{name}.bitis")));
    let dir = snapshot_dir().join(name);
    let update = std::env::var("BITIS_UPDATE_SNAPSHOTS").is_ok();
    if update { fs::create_dir_all(&dir).unwrap(); }

    let mut failures = vec![];
    for (file, content) in generate(name, &p) {
        let content = normalize(&content);
        let path = dir.join(&file);
        if update {
            fs::write(&path, &content).unwrap();
            continue;
        }
        let expected = fs::read_to_string(&path).unwrap_or_else(|_| panic!("missing snapshot {}", path.display()));
        if expected != content {
            let line = expected.lines().zip(content.lines()).position(|(a, b)| a != b)
                .unwrap_or(expected.lines().count().min(content.lines().count()));
            failures.push(format!("{} differs (first difference in line {}):\n--- expected\n{}\n+++ generated\n{}",
                                  path.display(), line + 1,
                                  expected.lines().nth(line).unwrap_or("<eof>"), content.lines().nth(line).unwrap_or("<eof>")));
        }
    }
    assert!(failures.is_empty(), "{}\n(run with BITIS_UPDATE_SNAPSHOTS=1 to update)", failures.join("\n\n"));
}

#[test]
fn snapshot_all_types() { check_snapshot("all_types"); }

#[test]
fn snapshot_docs() { check_snapshot("docs"); }

#[test]
fn snapshot_nested() { check_snapshot("nested"); }

/// Maximum lengths [M] (spec 2.16) in every target.
#[test]
fn snapshot_bounds() { check_snapshot("bounds"); }

#[test]
fn output_is_deterministic() {
    let p = compile_file(&snapshot_dir().join("all_types.bitis"));
    let first = generate("all_types", &p);
    for _ in 0..5 {
        let p = compile_file(&snapshot_dir().join("all_types.bitis"));
        assert_eq!(first, generate("all_types", &p));
    }
}

#[test]
fn doc_comments_are_valid_code() {
    let p = compile_file(&snapshot_dir().join("docs.bitis"));
    let rust = render_rust(&p).unwrap();
    assert!(rust.contains("/// Message doc\n#[derive(BiserdiMsg"), "{rust}");
    assert!(rust.contains("  /// multi\n  /// line attribute doc\n  pub kind: Kind,"), "{rust}");
    assert!(rust.contains("  /// first value\n  #[default] KindA,"), "{rust}");
    assert!(rust.contains("  /// element doc\n  Flag(bool),"), "{rust}");
    assert!(!rust.contains("dangling"), "{rust}");
    let cpp = render_cpp(&p, "bitis_lib.h", "docs").unwrap();
    assert!(cpp.contains("/// Message doc\nstruct Documented {"), "{cpp}");
    // a trailing backslash would continue the C++ comment on the next line
    assert!(!cpp.contains("\\\n"), "{cpp}");
    assert!(cpp.contains("<html> & \"quotes\""), "no html escaping: {cpp}");
    let py = render_python(&p, "docs").unwrap();
    assert!(py.pyi.contains("  \"\"\"\n  Message doc\n  \"\"\"\n"), "{}", py.pyi);
}

#[test]
fn schema_hash_independent_of_file_split_and_formatting() {
    let one = vec![SourceFile { name: "a.bitis".into(), content: fs::read_to_string(snapshot_dir().join("nested.bitis")).unwrap() }];
    let split = vec![
        SourceFile { name: "b.bitis".into(), content: "enum E(1) { *A, B, }\n//| doc\nmsg Leaf {uint_2 v;}".into() },
        SourceFile { name: "c.bitis".into(), content: "msg Middle { repeated_dyn_2 Leaf leaves; optional E e; }\n\
            msg Top { Middle m; oneof outer(2) { *Leaf leaf; oneof inner(1) { *bool x; E e; } } }".into() },
    ];
    let a = compile_sources(&one).unwrap();
    let b = compile_sources(&split).unwrap();
    assert_eq!(canonical_schema(&a), canonical_schema(&b));
    assert_eq!(schema_hash(&a), schema_hash(&b));
    assert_eq!(schema_hash_hex(&a), format!("{:016x}", schema_hash(&a)));
}

#[test]
fn schema_hash_of_wire_format_example() {
    // the example of docs/WIRE_FORMAT.md, section "Schema hash"
    let files = vec![SourceFile { name: "x.bitis".into(),
        content: "enum E(2) { *A, B }\nmsg M { optional uint_3 x; oneof o(2) { *bool b; E e; } }".into() }];
    let p = compile_sources(&files).unwrap();
    assert_eq!(canonical_schema(&p), "bitis-schema-v1\nenum E(2){*A,B}\nmsg M{optional uint_3 x;oneof o(2){*bool b;E e;};}\n");
    assert_eq!(schema_hash_hex(&p), "04ee18ca6b3375b1");
}

#[test]
fn dart_output() {
    let p = compile_file(&snapshot_dir().join("all_types.bitis"));
    let dart = render_dart(&p).unwrap();
    let imports: Vec<&str> = dart.lines().filter(|l| l.starts_with("import ")).collect();
    assert_eq!(imports, vec!["import 'dart:typed_data';", "import 'package:bitis/bitis.dart';"]);
    assert!(dart.contains("\n// dart format off\n"), "{dart}");
    assert!(dart.contains(&format!("const String bitisSchemaHashHex = '{}';", schema_hash_hex(&p))));
    let h = schema_hash(&p);
    assert!(dart.contains(&format!("Int64.fromInts(0x{:08x}, 0x{:08x});", h >> 32, h & 0xffff_ffff)));
    // type mapping and the width/range parameters passed to the runtime
    for s in ["final int u17;", "final BigInt u64V;", "final Int64 i33;", "w.writeUintBigInt(u64V, 64);",
              "w.writeIntInt64(i33, 33);", "w.writeDynInt(id, 16, 3);", "static const FixedPointType latFpType",
              "FixedPointType(34, -90, 90)", "final Uint8List data;", "final int? optU;",
              "rdInner: r.readDynArray(4, 3, Inner.bitisRead)", "rdColor: r.readDynArray(3, 1, Color.bitisRead)",
              "sealed class OO_AllTypes_Choice {", "final class OO_AllTypes_Choice_Flag extends OO_AllTypes_Choice {",
              "static const Color defaultValue = Color.green;", "blueLight('BLUE_LIGHT')",
              "this.fp = const FixedPoint.overflow()", "'opt_u': v"] {
        assert!(dart.contains(s), "missing {s:?} in\n{dart}");
    }
    // only enums: no dart:typed_data import (it would be unused)
    let files = vec![SourceFile { name: "e.bitis".into(), content: "enum E(1) { *A, B }".into() }];
    let dart = render_dart(&compile_sources(&files).unwrap()).unwrap();
    assert!(!dart.contains("dart:typed_data"), "{dart}");
}

#[test]
fn dart_bounds_and_max_bits() {
    let files = vec![SourceFile { name: "b.bitis".into(), content: "enum E(2) { *A, B, C }\n\
        msg B { astr_d4[5] s; repeated_dyn_2[3] astr_d4[2] l; optional binary_d3[4] o; \
                repeated_fixed_2 binary_d2[2] f; oneof c(1) { *astr_d3[4] t; E e; } }\n\
        msg U { astr_d4 s; }".into() }];
    let p = compile_sources(&files).unwrap();
    let dart = render_dart(&p).unwrap();
    let max = max_bits_message(&p.msgs[0], &p.msgs, &p.enums).unwrap();
    for s in [format!("static const int bitisMaxBits = {max};"), "static const int? bitisMaxBits = null;".to_string(),
              "s: r.readAStr(4, maxLength: 5)".into(), "w.writeAStr(s, 4, maxLength: 5);".into(),
              "l: r.readDynArray(2, 1, (r) => r.readAStr(4, maxLength: 2), maxLength: 3),".into(),
              "o: r.readOptional((r) => r.readBinary(3, maxLength: 4))".into(),
              "r.readFixedArray(2, (r) => r.readBinary(2, maxLength: 2))".into(),
              "OO_B_C_T(r.readAStr(3, maxLength: 4))".into(), "w.writeAStr(value, 3, maxLength: 4);".into(),
              "/// `repeated_dyn_2[3] astr_d4[2] l`".into(), "U(s: r.readAStr(4))".into()] {
        assert!(dart.contains(&s), "missing {s:?} in\n{dart}");
    }
}

#[test]
fn min_bits_per_spec() {
    let files = vec![SourceFile { name: "m.bitis".into(), content: "enum E(3) { *A, B }\n\
        msg Inner { uint_3 a; int_4 b; optional double c; repeated_fixed_2 bool d; repeated_dyn_2 uint_7 e; }\n\
        msg Outer { Inner i; E e; fp_9[0, 1] f; astr_d4 s; uint_32d4 u; int_8d2 v; float g;\n\
          oneof o(2) { *uint_12 x; Inner y; bool z; } }".into() }];
    let p = compile_sources(&files).unwrap();
    let inner = p.msgs.iter().find(|m| m.name == "Inner").unwrap();
    assert_eq!(min_bits_message(inner, &p.msgs), 3 + 5 + 1 + 2 + 1);
    let outer = p.msgs.iter().find(|m| m.name == "Outer").unwrap();
    assert_eq!(min_bits_message(outer, &p.msgs), 12 + 1 + 9 + 1 + 1 + 2 + 32 + (1 + 1));
}

#[test]
fn python_output() {
    let p = compile_file(&snapshot_dir().join("all_types.bitis"));
    let py = render_python(&p, "all_types").unwrap();
    // pyo3 0.29 API only (the compiler sets pyo3 = PYO3_VERSION in the Cargo.toml of the maturin project)
    for s in ["Python::with_gil", "PyObject", "PyException"] {
        assert!(!py.pyrust_rs.contains(s), "{s} in\n{}", py.pyrust_rs);
    }
    for s in ["pyo3::create_exception!(all_types, DecodeError, pyo3::exceptions::PyValueError,",
              "pub fn encode_error(kind: &str, detail: &str) -> PyErr {",
              "#[pyclass(eq, eq_int, hash, frozen, from_py_object)]",
              // a binary element of a oneof is printed with {:?} (Vec<u8> has no Display)
              "OO_AllTypes_Choice::Raw(v) => { write!(f, \"OO_AllTypes_Choice(Raw({:?}))\", v) },"] {
        assert!(py.pyrust_rs.contains(s), "missing {s:?} in\n{}", py.pyrust_rs);
    }
    assert!(py.lib_rs.contains("m.add(\"DecodeError\", m.py().get_type::<bitis_py_errors::DecodeError>())?;"), "{}", py.lib_rs);
    // stub: real attribute names and Python types
    for s in ["  u64v: int\n", "  s: str\n", "  data: bytes\n", "  opt_u: int | None\n", "  opt_inner: Inner | None\n",
              "  rf_inner: list[Inner]\n", "  rd_color: list[Color]\n", "  BlueLight: _typing.ClassVar[Color]\n",
              "  def serialize(self) -> tuple[bytes, int, int]:", "  def try_deserialize(data: bytes) -> tuple[AllTypes, int, int]:",
              "  def choice_raw(self) -> bytes | None: ...", "  def choice_oo(self) -> OO_AllTypes_ChoiceEnum:",
              "class DecodeError(ValueError):", "  # fp_10[-2,3]; above 3: Overflow, read back as 4.0; below -2 or NaN: Underflow, read back as -3.0\n"] {
        assert!(py.pyi.contains(s), "missing {s:?} in\n{}", py.pyi);
    }
    for s in ["String", "Vec<", "u64_v"] {
        assert!(!py.pyi.contains(s), "{s} in\n{}", py.pyi);
    }
}
