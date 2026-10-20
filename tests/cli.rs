//! Tests of the `bitis` command line tool. All files are written into temporary directories
//! below the cargo target directory.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Fresh, empty temporary directory.
fn temp_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("cli_{}_{}_{}", name, std::process::id(), COUNTER.fetch_add(1, Ordering::SeqCst)));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Runs bitis in `dir`; kills it and fails if it does not finish within 20 s.
fn bitis(dir: &Path, args: &[&str]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_bitis")).args(args).current_dir(dir)
        .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if child.try_wait().unwrap().is_some() { break; }
        if Instant::now() > deadline {
            let _ = child.kill();
            panic!("bitis {args:?} did not terminate (hang)");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    child.wait_with_output().unwrap()
}
fn stdout(o: &Output) -> String { String::from_utf8_lossy(&o.stdout).to_string() }
fn stderr(o: &Output) -> String { String::from_utf8_lossy(&o.stderr).to_string() }

const SCHEMA: &str = "//| doc\nmsg Mm {\n  uint_3 a;\n  fp_8[0, 10] f;\n}\n";

#[test]
fn cpp_bare_output_file_writes_header_lib_next_to_it() {
    let dir = temp_dir("cpp_bare");
    fs::write(dir.join("schema.bitis"), SCHEMA).unwrap();
    let o = bitis(&dir, &["compile", "-l", "cpp", "-i", "schema.bitis", "-o", "messages.h"]);
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(dir.join("messages.h").is_file());
    assert!(dir.join("bitis_lib.h").is_file());
    let h = fs::read_to_string(dir.join("messages.h")).unwrap();
    assert!(h.contains("#include \"bitis_lib.h\""));
    assert!(h.contains("namespace bitis_schema_messages {"), "{h}");
    // no generated code / debug output without -d
    assert_eq!(stdout(&o), "");
}

#[test]
fn cpp_header_lib_file_name_is_used_for_include_and_written_file() {
    let dir = temp_dir("cpp_lib_name");
    fs::create_dir_all(dir.join("out")).unwrap();
    fs::write(dir.join("schema.bitis"), SCHEMA).unwrap();
    let o = bitis(&dir, &["compile", "-l", "cpp", "-i", "schema.bitis", "-o", "out", "--bitis-header-lib-file-name", "my_bitis.h"]);
    assert!(o.status.success(), "{}", stderr(&o));
    let h = fs::read_to_string(dir.join("out/schema.h")).unwrap();
    assert!(h.contains("#include \"my_bitis.h\""));
    assert!(dir.join("out/my_bitis.h").is_file());
    assert!(!dir.join("out/bitis_lib.h").exists());

    let o = bitis(&dir, &["compile", "-l", "cpp", "-i", "schema.bitis", "-o", "out/x.h", "--prevent-write-bitis-header-lib"]);
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(dir.join("out/x.h").is_file());
    assert!(!dir.join("out/bitis_lib.h").exists());
}

#[test]
fn rust_output_locations() {
    let dir = temp_dir("rust_out");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("defs")).unwrap();
    fs::write(dir.join("defs/a.bitis"), "msg Aa { Bb b; }\n").unwrap();
    fs::write(dir.join("defs/b.bitis"), "msg Bb { bool x; }\n").unwrap();
    // several input files into a directory -> messages.rs
    let o = bitis(&dir, &["-d", "compile", "-l", "rust", "-o", "src", "-i", "defs/a.bitis", "-i", "defs/b.bitis"]);
    assert!(o.status.success(), "{}", stderr(&o));
    let rs = fs::read_to_string(dir.join("src/messages.rs")).unwrap();
    assert!(rs.contains("pub struct Aa {") && rs.contains("pub struct Bb {"));
    assert!(rs.contains("pub const BITIS_SCHEMA_HASH: u64 = 0x"));
    // single input file without -o -> next to the input file
    let o = bitis(&dir, &["compile", "-l", "rust", "-i", "defs/b.bitis"]);
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(dir.join("defs/b.rs").is_file());
    // several input files without -o -> error
    let o = bitis(&dir, &["compile", "-l", "rust", "-i", "defs/a.bitis", "defs/b.bitis"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(stderr(&o).contains("output file or path (-o) is required"), "{}", stderr(&o));
}

#[test]
fn errors_are_reported_with_position_and_exit_code_1() {
    let dir = temp_dir("errors");
    fs::write(dir.join("ok.bitis"), "msg Aa { bool x; }\n").unwrap();
    fs::write(dir.join("bad.bitis"), "msg Bb {\n  bool x;\n  Unknown u;\n  uint_3 x;\n}\n").unwrap();
    let o = bitis(&dir, &["compile", "-l", "rust", "-o", "out.rs", "-i", "ok.bitis", "-i", "bad.bitis"]);
    assert_eq!(o.status.code(), Some(1));
    assert_eq!(stderr(&o), "bad.bitis:3:3: error: unknown type 'Unknown' of attribute 'u' in message 'Bb'\n\
                            bad.bitis:4:10: error: duplicate attribute 'x' in message 'Bb' (first defined at bad.bitis:2:8)\n");
    assert_eq!(stdout(&o), "");
    assert!(!dir.join("out.rs").exists());
}

#[test]
fn user_errors_do_not_panic() {
    let dir = temp_dir("no_panic");
    fs::write(dir.join("noext"), "msg Aa { bool x; }\n").unwrap();
    fs::write(dir.join("a.bitis"), "msg Aa { bool x; }\n").unwrap();
    for args in [
        vec!["compile", "-l", "rust", "-i", "noext", "-o", "x.rs"],
        vec!["compile", "-l", "rust", "-i", "missing.bitis", "-o", "x.rs"],
        vec!["compile", "-l", "cpp", "-i", "a.bitis", "-o", "nodir/x.h"],
        vec!["compile", "-l", "python", "-i", "a.bitis"],
        vec!["compile", "-l", "python", "-i", "a.bitis", "-o", "missing_dir"],
        vec!["compile", "-l", "rust", "-i", "a.bitis", "-p", "missing_dir"],
    ] {
        let o = bitis(&dir, &args);
        assert_eq!(o.status.code(), Some(1), "{args:?}: {}", stderr(&o));
        assert!(stderr(&o).contains("error:"), "{args:?}: {}", stderr(&o));
        assert!(!stderr(&o).contains("panicked"), "{args:?}: {}", stderr(&o));
    }
}

#[test]
fn former_hangs_terminate() {
    let dir = temp_dir("hangs");
    fs::write(dir.join("eof_oneof.bitis"), "msg Mm { oneof oo(2) { *bool aa;").unwrap();
    fs::write(dir.join("recursive.bitis"), "msg Aa { Bb b; }\nmsg Bb { optional Aa a; }\n").unwrap();
    for (file, expected) in [
        ("eof_oneof.bitis", "eof_oneof.bitis:1:33: error: unexpected end of file"),
        ("recursive.bitis", "recursive.bitis:2:19: error: recursive message definition: Aa -> Bb -> Aa"),
    ] {
        for lang in ["rust", "cpp"] {
            let o = bitis(&dir, &["compile", "-l", lang, "-i", file, "-o", "out"]);
            assert_eq!(o.status.code(), Some(1));
            assert!(stderr(&o).starts_with(expected), "{}", stderr(&o));
        }
    }
}

#[test]
fn debug_output_only_with_debug_flag() {
    let dir = temp_dir("debug");
    fs::write(dir.join("schema.bitis"), SCHEMA).unwrap();
    let o = bitis(&dir, &["compile", "-l", "rust", "-i", "schema.bitis", "-o", "s.rs"]);
    assert!(o.status.success());
    assert_eq!(stdout(&o), "");
    let o = bitis(&dir, &["-d", "compile", "-l", "rust", "-i", "schema.bitis", "-o", "s.rs"]);
    assert!(o.status.success());
    assert!(stdout(&o).contains("FixedPoint summary"), "{}", stdout(&o));
    assert!(stdout(&o).contains("Schema hash: "), "{}", stdout(&o));
    assert!(!stdout(&o).contains("pub struct Mm"), "generated code must not be printed: {}", stdout(&o));
}
