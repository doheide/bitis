//! Snapshot tests of the generated code.
//!
//! For every `tests/snapshots/<name>.bitis` the generated Rust, C++ and Python files are compared with
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

fn normalize(s: &str) -> String { s.replace(&format!("\"{}\"", BITIS_COMPILER_VERSION), "\"{BITIS_VERSION}\"") }

fn generate(name: &str, p: &BitisProcessed) -> Vec<(String, String)> {
    let py = render_python(p, name).unwrap();
    vec![
        (format!("{name}.rs"), render_rust(p).unwrap()),
        (format!("{name}.h"), render_cpp(p, "bitis_lib.h", &stamp_name_from(name)).unwrap()),
        ("py_messages.rs".to_string(), py.messages_rs),
        ("py_pyrust.rs".to_string(), py.pyrust_rs),
        ("py_lib.rs".to_string(), py.lib_rs),
        (format!("{name}.pyi"), py.pyi),
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
