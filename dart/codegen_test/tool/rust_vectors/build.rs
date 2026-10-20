//! Generates `FromJ` implementations (JSON value -> Rust value, using the JSON mapping of the
//! Dart target) and a dispatch function for every message of the generated modules `src/gen/*.rs`.
//! The generated Rust code is regular enough to be read line by line.
use std::fmt::Write as _;
use std::{env, fs, path::Path};

fn main() {
    let out = env::var("OUT_DIR").unwrap();
    println!("cargo:rerun-if-changed=src/gen");
    let mut entries: Vec<_> = fs::read_dir("src/gen").expect("src/gen missing: run ../regen.sh --vectors")
        .map(|e| e.unwrap().path()).filter(|p| p.extension().is_some_and(|e| e == "rs")).collect();
    entries.sort();
    for path in entries {
        println!("cargo:rerun-if-changed={}", path.display());
        let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
        let glue = glue(&fs::read_to_string(&path).unwrap());
        fs::write(Path::new(&out).join(format!("glue_{stem}.rs")), glue).unwrap();
    }
}

/// Lower case without underscores: matches the schema name (`CPU`, `std_msg_ip`) with the
/// PascalCase Rust name (`Cpu`, `StdMsgIp`).
fn norm(s: &str) -> String { s.chars().filter(|c| *c != '_').flat_map(|c| c.to_lowercase()).collect() }

fn glue(src: &str) -> String {
    let mut out = String::new();
    let mut msgs = vec![];
    let mut lines = src.lines().peekable();
    let mut last_derive = String::new();
    while let Some(line) = lines.next() {
        let t = line.trim();
        if t.starts_with("#[derive(") { last_derive = t.to_string(); continue; }
        if let Some(rest) = t.strip_prefix("pub enum ") {
            let name = rest.split_whitespace().next().unwrap().to_string();
            let is_oneof = last_derive.contains("BiserdiOneOf");
            let mut arms = String::new();
            for l in lines.by_ref() {
                let l = l.trim();
                if l == "}" { break; }
                if l.starts_with("///") || l.is_empty() { continue; }
                let l = l.trim_start_matches("#[default]").trim();
                let variant: String = l.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
                if is_oneof {
                    let _ = write!(arms, "            \"{}\" => Self::{variant}(FromJ::from_j(v)),\n", norm(&variant));
                } else {
                    let _ = write!(arms, "            \"{}\" => Self::{variant},\n", norm(&variant));
                }
            }
            if is_oneof {
                let _ = write!(out, "impl FromJ for {name} {{\n    fn from_j(j: &J) -> Self {{\n        let (k, v) = oneof_entry(j);\n        match norm(k).as_str() {{\n{arms}            k => panic!(\"unknown element {{k}} of {name}\"),\n        }}\n    }}\n}}\n");
            } else {
                let _ = write!(out, "impl FromJ for {name} {{\n    fn from_j(j: &J) -> Self {{\n        let k = j.as_str().expect(\"enum value must be a string\");\n        match norm(k).as_str() {{\n{arms}            k => panic!(\"unknown value {{k}} of {name}\"),\n        }}\n    }}\n}}\n");
            }
        } else if let Some(rest) = t.strip_prefix("pub struct ") {
            let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            let mut fields = String::new();
            for l in lines.by_ref() {
                let l = l.trim();
                if l == "}" { break; }
                if let Some(f) = l.strip_prefix("pub ") {
                    let f = f.split(':').next().unwrap().trim();
                    let _ = write!(fields, "            {f}: FromJ::from_j(field(j, \"{f}\")),\n");
                }
            }
            let _ = write!(out, "#[allow(unused_variables)]\nimpl FromJ for {name} {{\n    fn from_j(j: &J) -> Self {{\n        Self {{\n{fields}        }}\n    }}\n}}\n");
            msgs.push(name);
        }
    }
    out.push_str("/// Encodes the JSON value (or the default value if `None`) of the message `name`.\n");
    out.push_str("pub fn encode(name: &str, j: Option<&J>) -> Option<Encoded> {\n    Some(match name {\n");
    for m in &msgs { let _ = writeln!(out, "        \"{m}\" => encode_as::<{m}>(j),"); }
    out.push_str("        _ => return None,\n    })\n}\n");
    let _ = writeln!(out, "/// All messages in declaration order.\npub const MESSAGES: &[&str] = &[{}];",
                     msgs.iter().map(|m| format!("\"{m}\"")).collect::<Vec<_>>().join(", "));
    out
}
