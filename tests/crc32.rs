//! CRC-32 helper against the shared vectors golden/vectors/crc32.txt (also read by the
//! C++ and Dart tests, see golden/README.md): one-shot, incremental with several splits,
//! byte by byte, reset.

use std::path::Path;

use bitis_lib::{crc32, Crc32};

struct Vector { name: String, data: Vec<u8>, crc: u32 }

fn unhex(s: &str) -> Vec<u8> {
    if s == "-" { return Vec::new(); }
    assert!(s.len() % 2 == 0, "odd hex length");
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex")).collect()
}

fn vectors() -> Vec<Vector> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("golden/vectors/crc32.txt");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut out = Vec::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') { continue; }
        let f: Vec<&str> = line.split(' ').collect();
        assert_eq!(f.len(), 4, "bad line: {line}");
        let data = match f[0] {
            "hex" => unhex(f[2]),
            "ascii" => f[2].as_bytes().to_vec(),
            k => panic!("unknown record kind {k}"),
        };
        let crc = u32::from_str_radix(f[3], 16).expect("crc hex");
        out.push(Vector { name: f[1].to_string(), data, crc });
    }
    assert!(out.len() >= 16, "too few vectors");
    out
}

#[test]
fn crc32_vectors() {
    for v in vectors() {
        let d = &v.data;
        assert_eq!(crc32(d), v.crc, "{}: one-shot", v.name);

        // splits at the start, the end, the middle and every position of short inputs
        let mut cuts: Vec<usize> = vec![0, 1, d.len() / 2, d.len().saturating_sub(1), d.len()];
        if d.len() <= 64 { cuts.extend(0..=d.len()); }
        for &i in cuts.iter().filter(|&&i| i <= d.len()) {
            let mut c = Crc32::new();
            c.update(&d[..i]);
            c.update(&d[i..]);
            assert_eq!(c.finish(), v.crc, "{}: split at {i}", v.name);
        }
        // three pieces, with empty updates in between
        let (a, b) = (d.len() / 3, 2 * d.len() / 3);
        let mut c = Crc32::new();
        for piece in [&d[..a], &[][..], &d[a..b], &d[b..], &[][..]] { c.update(piece); }
        assert_eq!(c.finish(), v.crc, "{}: three pieces", v.name);
        // byte by byte, finish() in between does not change the state
        let mut c = Crc32::default();
        for (i, byte) in d.iter().enumerate() {
            c.update(std::slice::from_ref(byte));
            if i == d.len() / 2 { assert_eq!(c.finish(), crc32(&d[..=i])); }
        }
        assert_eq!(c.finish(), v.crc, "{}: byte by byte", v.name);
        // reset
        c.reset();
        assert_eq!(c.finish(), 0, "{}: reset", v.name);
        c.update(d);
        assert_eq!(c.finish(), v.crc, "{}: after reset", v.name);
    }
}

#[test]
fn crc32_vectors_content() {
    let vs = vectors();
    let get = |n: &str| vs.iter().find(|v| v.name == n).unwrap_or_else(|| panic!("missing {n}"));
    assert_eq!(get("check").data, b"123456789");
    assert_eq!(get("check").crc, 0xCBF4_3926);
    assert_eq!(get("empty").data, b"");
    assert_eq!(get("empty").crc, 0);
    // lcg_1000 follows the generation rule documented in the file
    let mut x: u32 = 0x1234_5678;
    let lcg: Vec<u8> = (0..1000).map(|_| { x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223); (x >> 24) as u8 }).collect();
    assert_eq!(get("lcg_1000").data, lcg);
}
