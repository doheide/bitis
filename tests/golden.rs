//! Golden vector tests of the Rust runtime (see golden/README.md).
//!
//! Reads golden/vectors/manifest.txt (shared with the C++ and Dart tests) and checks
//! - `valid`: decode (default and exact) consumes `total_bits`, re-encoding gives the same
//!   bytes; with appended bytes the default decoder still succeeds (same bits), the exact
//!   decoder fails with `TrailingData`,
//! - `noncanon`: decoding succeeds with `total_bits`, re-encoding gives the canonical bytes,
//! - `invalid`: decoding fails with the given error kind,
//! - spot checks of decoded field values and of the encoder for a few cases,
//! - the generated code in golden/gen/ is up to date with the compiler.

#[path = "../golden/gen/rust/golden.rs"]
#[allow(unused_imports)]
mod golden;

use std::fs;
use std::path::{Path, PathBuf};

use bitis_lib::*;
use golden::*;

fn golden_dir() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("golden") }

// ****************************************************************************
// manifest

#[derive(Debug, Clone)]
enum Record {
    Valid { case: String, ty: String, bits: u64, data: Vec<u8> },
    NonCanon { case: String, ty: String, bits: u64, data: Vec<u8>, canonical: Vec<u8> },
    Invalid { case: String, ty: String, error: DecodeError, exact: bool, data: Vec<u8> },
}

fn unhex(s: &str) -> Vec<u8> {
    if s == "-" { return Vec::new(); }
    assert!(s.len() % 2 == 0, "odd hex length");
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex")).collect()
}
fn hex(d: &[u8]) -> String {
    if d.is_empty() { return "-".into(); }
    d.iter().map(|b| format!("{:02x}", b)).collect()
}
fn parse_error(s: &str) -> DecodeError {
    match s {
        "UnexpectedEnd" => DecodeError::UnexpectedEnd,
        "InvalidValue" => DecodeError::InvalidValue,
        "InvalidDiscriminant" => DecodeError::InvalidDiscriminant,
        "TrailingData" => DecodeError::TrailingData,
        _ => panic!("unknown error kind '{s}'"),
    }
}

fn manifest() -> Vec<Record> {
    let text = fs::read_to_string(golden_dir().join("vectors").join("manifest.txt")).expect("manifest");
    let mut recs = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        let f: Vec<&str> = line.split(' ').collect();
        fn malformed<T>(n: usize, line: &str) -> T { panic!("manifest line {}: malformed: {line}", n + 1) }
        match f[0] {
            "valid" if f.len() == 5 => recs.push(Record::Valid { case: f[1].into(), ty: f[2].into(),
                bits: f[3].parse().unwrap_or_else(|_| malformed(n, line)), data: unhex(f[4]) }),
            "noncanon" if f.len() == 6 => recs.push(Record::NonCanon { case: f[1].into(), ty: f[2].into(),
                bits: f[3].parse().unwrap_or_else(|_| malformed(n, line)), data: unhex(f[4]), canonical: unhex(f[5]) }),
            "invalid" if f.len() == 6 => recs.push(Record::Invalid { case: f[1].into(), ty: f[2].into(),
                error: parse_error(f[3]), exact: match f[4] { "exact" => true, "default" => false, _ => malformed(n, line) },
                data: unhex(f[5]) }),
            _ => malformed(n, line),
        }
    }
    recs
}
fn case_bytes(case: &str) -> Vec<u8> {
    for r in manifest() {
        if let Record::Valid { case: c, data, .. } | Record::NonCanon { case: c, data, .. } = r {
            if c == case { return data; }
        }
    }
    panic!("no case {case}");
}

// ****************************************************************************
// decoding by type name

/// `(re-encoded bytes, consumed bits)` of the value decoded from `data`.
fn roundtrip<T: BiserdiTrait>(data: &[u8], exact: bool) -> Result<(Vec<u8>, u64), DecodeError> {
    let (v, sizes) = if exact {
        let v = try_deserialize_exact::<T>(data)?;
        (v, try_deserialize::<T>(data)?.1)
    } else { try_deserialize::<T>(data)? };
    assert_eq!(sizes.total_bytes, sizes.total_bits.div_ceil(8));
    let (bytes, _) = try_serialize(&v).expect("re-encoding a decoded value must not fail");
    Ok((bytes, sizes.total_bits))
}
fn decode_by_name(ty: &str, data: &[u8], exact: bool) -> Result<(Vec<u8>, u64), DecodeError> {
    macro_rules! dispatch { ($($t:ident),*) => {
        match ty { $(stringify!($t) => roundtrip::<$t>(data, exact),)* _ => panic!("unknown message type '{ty}'") }
    } }
    dispatch!(Empty, Ints, DynInts, Floats, FixedPoints, Texts, Leaf, Mid, Nested, Containers, Enums, IntList,
        Choice, Everything)
}
fn decode<T: BiserdiTrait>(case: &str) -> T {
    try_deserialize_exact::<T>(&case_bytes(case)).unwrap_or_else(|e| panic!("{case}: {e}"))
}

fn check_all(filter: impl Fn(&Record) -> bool, check: impl Fn(&Record) -> Result<(), String>) {
    let mut n = 0;
    let mut failures = Vec::new();
    for r in manifest().iter().filter(|r| filter(r)) {
        n += 1;
        if let Err(e) = check(r) { failures.push(e); }
    }
    assert!(n > 0, "no records");
    assert!(failures.is_empty(), "{} of {n} golden vectors failed:\n{}", failures.len(), failures.join("\n"));
}

// ****************************************************************************
#[test]
fn golden_valid() {
    check_all(|r| matches!(r, Record::Valid { .. }), |r| {
        let Record::Valid { case, ty, bits, data } = r else { unreachable!() };
        for exact in [false, true] {
            match decode_by_name(ty, data, exact) {
                Err(e) => return Err(format!("{case}: decode (exact={exact}) failed: {e:?}")),
                Ok((re, b)) => {
                    if b != *bits { return Err(format!("{case}: total_bits {b}, expected {bits}")); }
                    if re != *data { return Err(format!("{case}: re-encoded {}, expected {}", hex(&re), hex(data))); }
                }
            }
        }
        if data.len() as u64 != bits.div_ceil(8) { return Err(format!("{case}: hex length does not match total_bits")); }
        // trailing bytes: ignored by the default decoder, rejected by the exact one
        let mut longer = data.clone();
        longer.extend_from_slice(&[0xa5, 0xff]);
        match decode_by_name(ty, &longer, false) {
            Ok((re, b)) if b == *bits && re == *data => {}
            other => return Err(format!("{case}: default decode with trailing bytes: {other:?}")),
        }
        match decode_by_name(ty, &longer, true) {
            Err(DecodeError::TrailingData) => Ok(()),
            other => Err(format!("{case}: exact decode with trailing bytes: {other:?}, expected TrailingData")),
        }
    });
}

#[test]
fn golden_noncanon() {
    check_all(|r| matches!(r, Record::NonCanon { .. }), |r| {
        let Record::NonCanon { case, ty, bits, data, canonical } = r else { unreachable!() };
        for exact in [false, true] {
            match decode_by_name(ty, data, exact) {
                Err(e) => return Err(format!("{case}: decode (exact={exact}) failed: {e:?}")),
                Ok((re, b)) => {
                    if b != *bits { return Err(format!("{case}: total_bits {b}, expected {bits}")); }
                    if re != *canonical {
                        return Err(format!("{case}: re-encoded {}, expected {}", hex(&re), hex(canonical)));
                    }
                }
            }
        }
        Ok(())
    });
}

#[test]
fn golden_invalid() {
    check_all(|r| matches!(r, Record::Invalid { .. }), |r| {
        let Record::Invalid { case, ty, error, exact, data } = r else { unreachable!() };
        match decode_by_name(ty, data, *exact) {
            Err(e) if e == *error => Ok(()),
            Err(e) => Err(format!("{case}: error {e:?}, expected {error:?}")),
            Ok((re, b)) => Err(format!("{case}: decoded ({b} bits, re-encoded {}), expected {error:?}", hex(&re))),
        }
    });
}

#[test]
fn golden_manifest_matches_schema() {
    let text = fs::read_to_string(golden_dir().join("vectors").join("manifest.txt")).unwrap();
    let stamp = format!("schema hash {:016x}", golden::BITIS_SCHEMA_HASH);
    assert!(text.contains(&stamp), "manifest was generated for another schema (expected '{stamp}')");
}

/// golden/gen/{rust,cpp} must be the current compiler output for golden/schema/golden.bitis.
#[test]
fn golden_generated_code_is_up_to_date() {
    let schema = golden_dir().join("schema").join("golden.bitis");
    let files = vec![SourceFile { name: "golden.bitis".into(), content: fs::read_to_string(&schema).unwrap() }];
    let p = compile_sources(&files).unwrap_or_else(|e| panic!("{}", e.iter().map(|e| e.render(&files)).collect::<Vec<_>>().join("\n")));
    let expected = [
        ("gen/rust/golden.rs", render_rust(&p).unwrap()),
        ("gen/cpp/golden.h", render_cpp(&p, "bitis_lib.h", &stamp_name_from("golden")).unwrap()),
    ];
    for (path, content) in expected {
        let committed = fs::read_to_string(golden_dir().join(path)).unwrap();
        assert!(committed == content, "golden/{path} is outdated, regenerate it (see golden/README.md)");
    }
}

// ****************************************************************************
// spot checks: decoded values

fn fp_close(v: Option<f64>, expected: f64, tol: f64) {
    let v = v.expect("fixed point value");
    assert!((v - expected).abs() <= tol, "{v} != {expected} (tol {tol})");
}

#[test]
fn golden_spot_check_decoded_values() {
    let m: Ints = decode("ints_min");
    assert_eq!((m.val_i1.val, m.val_i7.val, m.val_i16.val, m.val_i32.val), (-1, -127, -32767, -2147483647));
    assert_eq!((m.val_i33.val, m.val_i64.val), (-8589934591, -9223372036854775807));
    let m: Ints = decode("ints_max");
    assert_eq!((m.val_u33.val, m.val_u63.val, m.val_u64.val), ((1 << 33) - 1, u64::MAX >> 1, u64::MAX));
    let m: Ints = decode("noncanon_ints_negzero");
    assert_eq!((m.val_i1.val, m.val_i7.val, m.val_i16.val, m.val_i64.val), (0, 0, 0, 0));

    let m: DynInts = decode("dynints_min");
    assert_eq!((m.val_i16d5.val, m.val_i64d8.val), (-32767, -9223372036854775807));
    let m: DynInts = decode("dynints_max");
    assert_eq!((m.val_u8d3.val, m.val_u64d7.val), (255, u64::MAX));
    let m: DynInts = decode("noncanon_dynints_nonminimal");
    assert_eq!((m.val_u8d3.val, m.val_u32d4.val, m.val_i16d5.val), (5, 5, -3));

    let m: Floats = decode("floats_negzero");
    assert!(m.val_f64 == 0.0 && m.val_f64.is_sign_negative() && m.val_f32.is_sign_negative());
    let m: Floats = decode("floats_nan");
    assert!(m.val_f32.is_nan() && m.val_f64.is_nan());
    let m: Floats = decode("floats_subnormal");
    assert_eq!((m.val_f32.to_bits(), m.val_f64.to_bits()), (1, 1));

    let m: FixedPoints = decode("fixedpoints_gnss");
    fp_close(m.lat.value(), 51.756349, 6e-9);
    fp_close(m.lng.value(), 14.332868, 6e-9);
    // 0.49999999999999994 + 1.0 is 1.5 in f64 (spec 2.7: f64 arithmetic), then rounded away from zero
    fp_close(m.fp2.value(), 1.0, 0.0);
    fp_close(m.fp10.value(), -1.0, 0.0025);
    fp_close(m.fp53.value(), 123456.789, 1e-9);
    let m: FixedPoints = decode("fixedpoints_gnss_neg");
    fp_close(m.lat.value(), -33.856784, 6e-9);
    fp_close(m.lng.value(), -151.215297, 6e-9);
    let m: FixedPoints = decode("fixedpoints_min");
    assert_eq!([m.fp2.value(), m.fp10.value(), m.lat.value(), m.lng.value(), m.fp53.value()],
        [Some(0.0), Some(-2.0), Some(-90.0), Some(-180.0), Some(-1e6)]);
    let m: FixedPoints = decode("fixedpoints_max");
    assert_eq!([m.fp2.value(), m.fp10.value(), m.lat.value(), m.lng.value(), m.fp53.value()],
        [Some(1.0), Some(3.0), Some(90.0), Some(180.0), Some(1e6)]);
    let m: FixedPoints = decode("fixedpoints_default");
    assert!(m.fp2.is_overflow() && m.fp10.is_overflow() && m.lat.is_overflow() && m.lng.is_overflow() && m.fp53.is_overflow());
    let m: FixedPoints = decode("fixedpoints_underflow");
    assert!(m.fp2.is_underflow() && m.fp10.is_underflow() && m.lat.is_underflow() && m.lng.is_underflow() && m.fp53.is_underflow());
    let m: FixedPoints = decode("fixedpoints_zero");
    assert_eq!(m.fp2.value(), Some(1.0));   // 0.5 is a rounding tie (t = 1.5), rounds away from zero

    let m: Texts = decode("texts_values");
    assert_eq!(m.text.get_string(), "Hello, bitis!");
    assert_eq!(m.data.val, vec![0x00, 0x01, 0x7f, 0x80, 0xfe, 0xff]);
    let m: Texts = decode("texts_ascii_edges");
    assert_eq!(m.text.get_string(), "\0 ~\x7f");
    let m: Texts = decode("texts_long");
    assert_eq!((m.text.get_string().len(), m.data.val.len(), m.data.val[299]), (43, 300, 43));

    let m: Containers = decode("containers_full");
    assert_eq!(m.opt_color.val, Some(Color::Yellow));
    assert_eq!(m.opt_text.val.as_ref().map(|s| s.get_string()), Some(String::new()));
    assert_eq!(m.fixed_ints.val.iter().map(|v| v.val).collect::<Vec<_>>(), vec![-127, 0, 127]);
    assert_eq!((m.dyn_leaves.val.len(), m.dyn_leaves.val[2].val.val, m.dyn_leaves.val[0].tag.get_string()),
        (3, -63, "deep".to_string()));
    let m: Containers = decode("containers_default");
    assert!(m.opt_u7.val.is_none() && m.opt_leaf.val.is_none() && m.dyn_ints.val.is_empty());

    let m: Enums = decode("enums_all");
    assert_eq!((m.color, m.level, m.colors.val.len(), m.colors.val[5]), (Color::Yellow, Level::High, 6, Color::Yellow));
    let m: Enums = decode("enums_default");
    assert_eq!((m.color, m.level), (Color::Green, Level::Low));

    let m: Choice = decode("choice_text");
    assert!(matches!(&m.value, OO_Choice_Value::Text(s) if s.get_string() == "oneof text") && m.tail);
    let m: Choice = decode("choice_big");
    assert!(matches!(m.value, OO_Choice_Value::Big(v) if v.val == -2147483647));
    let m: Choice = decode("choice_list");
    assert!(matches!(&m.value, OO_Choice_Value::List(l) if l.items.val.iter().map(|v| v.val).collect::<Vec<_>>() == vec![-1, 0, 1, 127, -127]));
    let m: Choice = decode("choice_nothing");
    assert!(matches!(m.value, OO_Choice_Value::Nothing(_)) && m.tail);

    let m: Nested = decode("nested_deep");
    assert_eq!((m.mid.id.val, m.mid.leaf.val.val, m.mid.leaf.tag.get_string()), (99, -42, "level3".to_string()));
    assert_eq!(m.mid.extra.val.as_ref().map(|l| l.tag.get_string()), Some("extra".to_string()));
    assert!(m.after);

    let m: Everything = decode("everything_full");
    assert_eq!((m.tail.val, m.ints.val_i64.val, m.nested.mid.leaf.val.val), (5, i64::MAX, -42));
}

// ****************************************************************************
// spot checks: encoder (values set from physical / native values)

fn enc<T: BiserdiTrait>(v: &T) -> Vec<u8> { try_serialize(v).expect("encode").0 }

#[test]
fn golden_spot_check_encoder() {
    // fixed point quantization incl. exact rounding ties (round half away from zero)
    let ties = FixedPoints { fp2: 0.5.into(), fp10: (-1.845739471106758).into(), lat: (-1.5684636309743063e-05).into(),
        lng: (-1.5684636309743063e-05).into(), fp53: (-3.32365743815901e-07).into() };
    assert_eq!(enc(&ties), case_bytes("fixedpoints_ties"));
    let gnss = FixedPoints { fp2: 0.49999999999999994.into(), fp10: (-1.0).into(), lat: 51.756349.into(),
        lng: 14.332868.into(), fp53: 123456.789.into() };
    assert_eq!(enc(&gnss), case_bytes("fixedpoints_gnss"));
    let under = FixedPoints { fp2: f64::NAN.into(), fp10: (-2.001).into(), lat: (-90.000001).into(),
        lng: f64::NEG_INFINITY.into(), fp53: (-1000000.001).into() };
    assert_eq!(enc(&under), case_bytes("fixedpoints_underflow"));

    let ints = Ints { val_i1: (-1).into(), val_i7: (-127).into(), val_i16: (-32767).into(), val_i32: (-2147483647).into(),
        val_i33: (-8589934591).into(), val_i64: (-9223372036854775807).into(), ..Default::default() };
    assert_eq!(enc(&ints), case_bytes("ints_min"));
    let texts = Texts { text: BitisAString::try_from_str("Hello, bitis!").unwrap(),
        data: vec![0x00, 0x01, 0x7f, 0x80, 0xfe, 0xff].into() };
    assert_eq!(enc(&texts), case_bytes("texts_values"));
    assert_eq!(enc(&Empty {}), Vec::<u8>::new());

    // values outside the valid range are rejected by the checked encoder
    let bad = Ints { val_i64: i64::MIN.into(), ..Default::default() };
    assert_eq!(try_serialize(&bad).err(), Some(EncodeError::ValueOutOfRange));
    let bad = Ints { val_i16: i16::MIN.into(), ..Default::default() };
    assert_eq!(try_serialize(&bad).err(), Some(EncodeError::ValueOutOfRange));
    let bad = DynInts { val_i16d5: i16::MIN.into(), ..Default::default() };
    assert_eq!(try_serialize(&bad).err(), Some(EncodeError::ValueOutOfRange));
    let bad = Ints { val_u7: 128.into(), ..Default::default() };
    assert_eq!(try_serialize(&bad).err(), Some(EncodeError::ValueOutOfRange));
}
