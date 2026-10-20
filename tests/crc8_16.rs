//! CRC-8/AUTOSAR, CRC-16/IBM-3740 and the transmission CRC helpers against the shared
//! vectors golden/vectors/crc8_16.txt (also read by the C++, Dart and Python tests, see
//! golden/README.md): one-shot, incremental with several splits, byte by byte, reset; frames
//! with append / verify / strip (little-endian CRC-16 trailer) and corrupted frames.

use std::path::Path;

use bitis_lib::{
    append_transmission_crc, crc16_ibm3740, crc8_autosar, strip_transmission_crc, transmission_crc,
    transmission_crc_len, verify_transmission_crc, with_transmission_crc, Crc16Ibm3740, Crc8Autosar,
};

struct Vector { name: String, data: Vec<u8>, crc8: u8, crc16: u16 }
struct Frame { name: String, header: Vec<u8>, payload: Vec<u8>, trailer: Vec<u8> }

fn unhex(s: &str) -> Vec<u8> {
    if s == "-" { return Vec::new(); }
    assert!(s.len() % 2 == 0, "odd hex length");
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex")).collect()
}

fn load() -> (Vec<Vector>, Vec<Frame>) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("golden/vectors/crc8_16.txt");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let (mut vs, mut fs) = (Vec::new(), Vec::new());
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') { continue; }
        let f: Vec<&str> = line.split(' ').collect();
        assert_eq!(f.len(), 5, "bad line: {line}");
        let data = match f[0] {
            "hex" => unhex(f[2]),
            "ascii" => f[2].as_bytes().to_vec(),
            "frame" => {
                fs.push(Frame { name: f[1].into(), header: unhex(f[2]), payload: unhex(f[3]), trailer: unhex(f[4]) });
                continue;
            }
            k => panic!("unknown record kind {k}"),
        };
        assert_eq!((f[3].len(), f[4].len()), (2, 4), "bad line: {line}");
        let crc8 = u8::from_str_radix(f[3], 16).expect("crc8 hex");
        let crc16 = u16::from_str_radix(f[4], 16).expect("crc16 hex");
        vs.push(Vector { name: f[1].to_string(), data, crc8, crc16 });
    }
    assert!(vs.len() >= 16, "too few vectors");
    assert!(fs.len() >= 11, "too few frames");
    (vs, fs)
}

#[test]
fn crc8_16_vectors() {
    for v in load().0 {
        let d = &v.data;
        assert_eq!(crc8_autosar(d), v.crc8, "{}: crc8 one-shot", v.name);
        assert_eq!(crc16_ibm3740(d), v.crc16, "{}: crc16 one-shot", v.name);

        // splits at the start, the end, the middle and every position of short inputs
        let mut cuts: Vec<usize> = vec![0, 1, d.len() / 2, d.len().saturating_sub(1), d.len()];
        if d.len() <= 64 { cuts.extend(0..=d.len()); }
        for &i in cuts.iter().filter(|&&i| i <= d.len()) {
            let mut c8 = Crc8Autosar::new();
            let mut c16 = Crc16Ibm3740::new();
            c8.update(&d[..i]);
            c8.update(&d[i..]);
            c16.update(&d[..i]);
            c16.update(&d[i..]);
            assert_eq!(c8.finish(), v.crc8, "{}: crc8 split at {i}", v.name);
            assert_eq!(c16.finish(), v.crc16, "{}: crc16 split at {i}", v.name);
        }
        // three pieces, with empty updates in between
        let (a, b) = (d.len() / 3, 2 * d.len() / 3);
        let mut c8 = Crc8Autosar::new();
        let mut c16 = Crc16Ibm3740::new();
        for piece in [&d[..a], &[][..], &d[a..b], &d[b..], &[][..]] {
            c8.update(piece);
            c16.update(piece);
        }
        assert_eq!((c8.finish(), c16.finish()), (v.crc8, v.crc16), "{}: three pieces", v.name);
        // byte by byte, finish() in between does not change the state
        let mut c8 = Crc8Autosar::default();
        let mut c16 = Crc16Ibm3740::default();
        for (i, byte) in d.iter().enumerate() {
            c8.update(std::slice::from_ref(byte));
            c16.update(std::slice::from_ref(byte));
            if i == d.len() / 2 {
                assert_eq!(c8.finish(), crc8_autosar(&d[..=i]));
                assert_eq!(c16.finish(), crc16_ibm3740(&d[..=i]));
            }
        }
        assert_eq!((c8.finish(), c16.finish()), (v.crc8, v.crc16), "{}: byte by byte", v.name);
        // reset
        c8.reset();
        c16.reset();
        assert_eq!((c8.finish(), c16.finish()), (0x00, 0xFFFF), "{}: reset", v.name);
        c8.update(d);
        c16.update(d);
        assert_eq!((c8.finish(), c16.finish()), (v.crc8, v.crc16), "{}: after reset", v.name);
        // property of the CRC function itself (not used by the transmission rule, whose trailer
        // is little-endian): the CRC-16 over data + CRC big-endian is 0
        let mut with = d.clone();
        with.extend_from_slice(&v.crc16.to_be_bytes());
        assert_eq!(crc16_ibm3740(&with), 0, "{}: residue", v.name);
    }
}

#[test]
fn crc8_16_vectors_content() {
    let (vs, fs) = load();
    let get = |n: &str| vs.iter().find(|v| v.name == n).unwrap_or_else(|| panic!("missing {n}"));
    assert_eq!(get("check").data, b"123456789");
    assert_eq!((get("check").crc8, get("check").crc16), (0xDF, 0x29B1));
    assert_eq!(get("empty").data, b"");
    assert_eq!((get("empty").crc8, get("empty").crc16), (0x00, 0xFFFF));
    let mut x: u32 = 0x1234_5678;
    let lcg: Vec<u8> = (0..1000).map(|_| { x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223); (x >> 24) as u8 }).collect();
    assert_eq!(get("lcg_1000").data, lcg);
    // the frames follow the payload rule, and FRAMING.md section 5 is in the file
    for f in &fs {
        let p: Vec<u8> = (0..f.payload.len()).map(|i| (i * 37 + 1) as u8).collect();
        assert_eq!(f.payload, p, "{}", f.name);
    }
    let frame = |n: &str| fs.iter().find(|f| f.name == n).unwrap_or_else(|| panic!("missing {n}"));
    assert_eq!((frame("plain_1").header.as_slice(), frame("plain_1").trailer.as_slice()), (&[0x02][..], &[0x95][..]));
    assert_eq!((frame("plain_13").header.as_slice(), frame("plain_13").trailer.as_slice()), (&[0x3A][..], &[0xEF][..]));
    assert_eq!((frame("plain_32").header.as_slice(), frame("plain_32").trailer.as_slice()), (&[0x88, 0x01][..], &[0xF4, 0x3B][..]));
    assert_eq!(frame("plain_300").trailer, [0xC7, 0x15]);
    let ns: Vec<usize> = fs.iter().map(|f| f.header.len() + f.payload.len()).collect();
    for n in [1, 14, 15] { assert!(ns.contains(&n), "no frame with n = {n}"); }
}

#[test]
fn transmission_frames() {
    for f in load().1 {
        let frame: Vec<u8> = [f.header.as_slice(), f.payload.as_slice()].concat();
        let n = frame.len();
        assert_eq!(transmission_crc_len(n), f.trailer.len(), "{}: trailer length", f.name);
        let crc = transmission_crc(&frame);
        let trailer: Vec<u8> = if f.trailer.len() == 1 { vec![crc as u8] } else { crc.to_le_bytes().to_vec() };
        assert_eq!(trailer, f.trailer, "{}: transmission_crc", f.name);
        if n <= 14 {
            assert_eq!(f.trailer, [crc8_autosar(&frame)], "{}: crc8", f.name);
        } else {
            assert_eq!(f.trailer, crc16_ibm3740(&frame).to_le_bytes(), "{}: crc16 little-endian", f.name);
        }

        let full = with_transmission_crc(&frame);
        assert_eq!(full, [frame.as_slice(), f.trailer.as_slice()].concat(), "{}: with", f.name);
        let mut appended = frame.clone();
        append_transmission_crc(&mut appended);
        assert_eq!(appended, full, "{}: append", f.name);
        assert!(verify_transmission_crc(&full), "{}: verify", f.name);
        assert_eq!(strip_transmission_crc(&full), Some(frame.as_slice()), "{}: strip", f.name);

        // every single-bit error (and a wrong trailer) is detected
        for bit in 0..full.len() * 8 {
            let mut bad = full.clone();
            bad[bit / 8] ^= 1 << (bit % 8);
            assert!(!verify_transmission_crc(&bad), "{}: bit {bit}", f.name);
            assert_eq!(strip_transmission_crc(&bad), None);
        }
    }
}

#[test]
fn transmission_lengths() {
    // n = 14 / 15 boundary
    assert_eq!(transmission_crc_len(0), 1);
    assert_eq!(transmission_crc_len(14), 1);
    assert_eq!(transmission_crc_len(15), 2);
    assert_eq!(transmission_crc_len(usize::MAX), 2);
    // L = 15 / 16 / 17 on the receiving side
    let l15 = with_transmission_crc(&[7u8; 14]);
    let l17 = with_transmission_crc(&[7u8; 15]);
    assert_eq!((l15.len(), l17.len()), (15, 17));
    assert!(verify_transmission_crc(&l15) && verify_transmission_crc(&l17));
    let mut l16 = vec![7u8; 14];
    l16.extend_from_slice(&crc16_ibm3740(&l16).to_le_bytes()); // would match as CRC-16
    assert_eq!(l16.len(), 16);
    assert!(!verify_transmission_crc(&l16));
    let mut l16b = vec![7u8; 15];
    l16b.push(crc8_autosar(&l16b)); // would match as CRC-8
    assert!(!verify_transmission_crc(&l16b));
    assert_eq!(strip_transmission_crc(&l16b), None);
    // empty input / n = 0
    assert!(!verify_transmission_crc(&[]));
    assert_eq!(strip_transmission_crc(&[]), None);
    assert_eq!(with_transmission_crc(&[]), [0x00]);
    assert_eq!(strip_transmission_crc(&[0x00]), Some(&[][..]));
    assert!(!verify_transmission_crc(&[0x01]));
}
