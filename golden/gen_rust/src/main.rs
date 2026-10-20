//! Generator of the bitis golden vectors (golden/vectors/manifest.txt).
//!
//! The Rust runtime is the reference: valid vectors are the bytes produced by
//! `try_serialize`. Invalid and non-canonical inputs are built bit by bit with
//! an independent writer (`Bits`, a direct transcription of docs/WIRE_FORMAT.md)
//! together with the error kind the SPEC demands; the generator then checks
//! that the Rust runtime agrees and aborts otherwise (a divergence has to be
//! resolved, not recorded).
//!
//! Usage (from the repository root):
//!   cargo run --manifest-path golden/gen_rust/Cargo.toml             # write the manifest
//!   cargo run --manifest-path golden/gen_rust/Cargo.toml -- --check  # fail if it is outdated

#[path = "../../gen/rust/golden.rs"]
#[allow(unused_imports)]
mod golden;

use std::collections::{BTreeMap, HashSet};
use std::fmt::Write as _;

use bitis_lib::*;
use golden::*;

const MANIFEST_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../vectors/manifest.txt");

// ****************************************************************************
// independent bit writer (spec section 1 and 2)

#[derive(Clone, Default, Debug)]
struct Bits { bits: Vec<bool> }

impl Bits {
    fn new() -> Self { Self::default() }
    /// The first `total_bits` bits of `data`.
    fn from_bytes(data: &[u8], total_bits: u64) -> Self {
        let mut b = Self::new();
        for i in 0..total_bits as usize { b.bits.push((data[i / 8] >> (i % 8)) & 1 == 1); }
        b
    }
    /// `W(v, w)`: the low `w` bits of `v`, LSB first.
    fn u(mut self, v: u64, w: u32) -> Self {
        for k in 0..w { self.bits.push(k < 64 && (v >> k) & 1 == 1); }
        self
    }
    fn b(self, v: bool) -> Self { self.u(v as u64, 1) }
    /// `int_X`: sign bit, then the magnitude with `X` bits.
    fn sm(self, negative: bool, magnitude: u64, x: u32) -> Self { self.b(negative).u(magnitude, x) }
    /// `uint_XdY` (spec 2.4), minimal encoding.
    fn dyn_u(self, v: u64, x: u32, y: u32) -> Self {
        if v == 0 { return self.b(false); }
        let mut s = self.b(true);
        let mut v = v;
        let mut remaining = x;
        loop {
            let c = y.min(remaining);
            s = s.u(v, c);
            v = if c >= 64 { 0 } else { v >> c };
            remaining -= c;
            if remaining == 0 { break; }
            s = s.b(v != 0);
            if v == 0 { break; }
        }
        s
    }
    /// `uint_XdY` written with the given chunks (allows non-minimal encodings, e.g. zero
    /// chunks after a continuation bit 1).
    fn dyn_chunks(self, chunks: &[u64], x: u32, y: u32) -> Self {
        let mut s = self.b(true);
        let mut remaining = x;
        for (i, c) in chunks.iter().enumerate() {
            let w = y.min(remaining);
            s = s.u(*c, w);
            remaining -= w;
            if remaining == 0 { assert_eq!(i + 1, chunks.len()); break; }
            s = s.b(i + 1 < chunks.len());
        }
        s
    }
    /// `int_XdY` (spec 2.5).
    fn dyn_i(self, negative: bool, magnitude: u64, x: u32, y: u32) -> Self { self.b(negative).dyn_u(magnitude, x, y) }
    /// `U32D(Y)` count followed by bytes.
    fn len(&self) -> u64 { self.bits.len() as u64 }
    /// Padded with zero bits to whole bytes.
    fn to_bytes(&self) -> Vec<u8> {
        let mut out = vec![0_u8; self.bits.len().div_ceil(8)];
        for (i, b) in self.bits.iter().enumerate() { if *b { out[i / 8] |= 1 << (i % 8); } }
        out
    }
}

// ****************************************************************************
// decoding by type name (same dispatch as tests/golden.rs)

/// Decodes `data` as `T` (exact mode: `try_deserialize_exact`), re-encodes the value and
/// returns `(re-encoded bytes, consumed bits)`.
fn roundtrip<T: BiserdiTrait>(data: &[u8], exact: bool) -> Result<(Vec<u8>, u64), DecodeError> {
    let (v, sizes) = try_deserialize::<T>(data)?;
    if exact { try_deserialize_exact::<T>(data)?; }
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

fn hex(d: &[u8]) -> String {
    if d.is_empty() { return "-".into(); }
    d.iter().map(|b| format!("{:02x}", b)).collect()
}
fn err_name(e: DecodeError) -> &'static str {
    match e {
        DecodeError::UnexpectedEnd => "UnexpectedEnd",
        DecodeError::InvalidValue => "InvalidValue",
        DecodeError::InvalidDiscriminant => "InvalidDiscriminant",
        DecodeError::TrailingData => "TrailingData",
    }
}

// ****************************************************************************
struct Gen {
    lines: Vec<String>,
    names: HashSet<String>,
    valid: BTreeMap<String, (String, Vec<u8>, u64)>,
}

impl Gen {
    fn name(&mut self, case: &str) {
        assert!(case.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'), "bad case name {case}");
        assert!(self.names.insert(case.to_string()), "duplicate case name {case}");
    }
    fn section(&mut self, title: &str) { self.lines.push(format!("# --- {title}")); }

    /// A valid value: encoded by the reference runtime; decoding (default and exact) and
    /// re-encoding must reproduce the bytes.
    fn valid<T: BiserdiTrait>(&mut self, case: &str, ty: &str, v: &T) {
        self.name(case);
        let (bytes, sizes) = try_serialize(v).unwrap_or_else(|e| panic!("{case}: encoding failed: {e}"));
        assert_eq!(sizes.total_bytes, bytes.len() as u64);
        for exact in [false, true] {
            let (re, bits) = decode_by_name(ty, &bytes, exact)
                .unwrap_or_else(|e| panic!("{case}: decoding the reference encoding failed: {e}"));
            assert_eq!(bits, sizes.total_bits, "{case}: consumed bits");
            assert_eq!(re, bytes, "{case}: re-encoding differs");
        }
        self.lines.push(format!("valid {case} {ty} {} {}", sizes.total_bits, hex(&bytes)));
        self.valid.insert(case.to_string(), (ty.to_string(), bytes, sizes.total_bits));
    }

    /// A valid but non-canonical input: decodes with `total_bits` bits, re-encodes to the
    /// canonical bytes of the value.
    fn noncanon(&mut self, case: &str, ty: &str, input: &Bits, canonical: &[u8]) {
        self.noncanon_bytes(case, ty, &input.to_bytes(), input.len(), canonical)
    }
    fn noncanon_bytes(&mut self, case: &str, ty: &str, input: &[u8], total_bits: u64, canonical: &[u8]) {
        self.name(case);
        for exact in [false, true] {
            let (re, bits) = decode_by_name(ty, input, exact)
                .unwrap_or_else(|e| panic!("{case}: Rust runtime rejects valid non-canonical input: {e}"));
            assert_eq!(bits, total_bits, "{case}: consumed bits");
            assert_eq!(re, canonical, "{case}: canonical re-encoding");
        }
        assert_ne!(input, canonical, "{case}: input is canonical");
        self.lines.push(format!("noncanon {case} {ty} {total_bits} {} {}", hex(input), hex(canonical)));
    }

    /// An invalid input and the error the spec requires.
    fn invalid(&mut self, case: &str, ty: &str, expected: DecodeError, exact: bool, input: &[u8]) {
        self.name(case);
        match decode_by_name(ty, input, exact) {
            Ok(_) => panic!("SPEC DIVERGENCE {case}: Rust runtime accepts the input, expected {expected:?}"),
            Err(e) => assert_eq!(e, expected, "SPEC DIVERGENCE {case}: Rust runtime error"),
        }
        if !exact && expected != DecodeError::TrailingData {
            // the exact decoder must report the same error
            assert_eq!(decode_by_name(ty, input, true).err(), Some(expected), "{case}: exact mode");
        }
        self.lines.push(format!("invalid {case} {ty} {} {} {}", err_name(expected),
            if exact { "exact" } else { "default" }, hex(input)));
    }
    fn invalid_bits(&mut self, case: &str, ty: &str, expected: DecodeError, input: &Bits) {
        self.invalid(case, ty, expected, false, &input.to_bytes())
    }
    /// Truncations of a valid vector to `lens` bytes (UnexpectedEnd).
    fn truncations(&mut self, valid_case: &str, lens: &[usize]) {
        let (ty, bytes, _) = self.valid.get(valid_case).unwrap_or_else(|| panic!("no case {valid_case}")).clone();
        for l in lens {
            assert!(*l < bytes.len(), "{valid_case}: {} bytes", bytes.len());
            self.invalid(&format!("trunc_{valid_case}_{l}"), &ty, DecodeError::UnexpectedEnd, false, &bytes[..*l]);
        }
    }
    fn valid_bytes(&self, case: &str) -> Vec<u8> { self.valid[case].1.clone() }
}

// ****************************************************************************
// value helpers

fn astr<const N: u8>(s: &str) -> BitisAString<N> { BitisAString::try_from_str(s).expect("ASCII") }
fn leaf(val: i16, tag: &str) -> Leaf { Leaf { val: IntWithGivenBitSize::try_new(val).unwrap(), tag: astr(tag) } }
fn some<T: BiserdiTrait + Default>(v: T) -> BitisOption<T> { BitisOption::new_some(v) }

// fixed point values with exact rounding ties t = k + 0.5 (k even) in the encoder
// (spec 2.7: round half away from zero), found by search in f64 arithmetic
const TIE_FP10: f64 = -1.845739471106758;      // t = 32.5
const TIE_LAT: f64 = -1.5684636309743063e-05;  // fp_34[-90,90]: t = 8589933094.5
const TIE_LNG: f64 = -1.5684636309743063e-05;  // fp_35[-180,180]: t = 17179867686.5
const TIE_FP53: f64 = -3.32365743815901e-07;   // fp_53[-1e6,1e6]: t = 4503599627368998.5

fn ints_max() -> Ints {
    Ints {
        flag: true, val_u1: 1.into(), val_u7: 127.into(), val_u32: u32::MAX.into(),
        val_u33: ((1_u64 << 33) - 1).into(), val_u63: (u64::MAX >> 1).into(), val_u64: u64::MAX.into(),
        val_i1: 1.into(), val_i7: 127.into(), val_i16: i16::MAX.into(), val_i32: i32::MAX.into(),
        val_i33: ((1_i64 << 33) - 1).into(), val_i64: i64::MAX.into(),
    }
}
fn dynints_max() -> DynInts {
    DynInts {
        val_u8d3: 255.into(), val_u32d4: u32::MAX.into(), val_u64d64: u64::MAX.into(), val_u64d7: u64::MAX.into(),
        val_i16d5: i16::MAX.into(), val_i64d8: i64::MAX.into(),
    }
}
fn fixedpoints_gnss() -> FixedPoints {
    FixedPoints { fp2: 0.49999999999999994.into(), fp10: (-1.0).into(), lat: 51.756349.into(),
        lng: 14.332868.into(), fp53: 123456.789.into() }
}
fn texts_values() -> Texts {
    Texts { text: astr("Hello, bitis!"), data: vec![0x00, 0x01, 0x7f, 0x80, 0xfe, 0xff].into() }
}
fn containers_full() -> Containers {
    Containers {
        opt_u7: some(127.into()), opt_leaf: some(leaf(-5, "x")), opt_text: some(astr("")),
        opt_color: some(Color::Yellow),
        fixed_ints: [(-127).into(), 0.into(), 127.into()].into(),
        fixed_leaves: [leaf(1, "a"), leaf(-1, "")].into(),
        dyn_ints: (0..10_u16).map(|v| v.into()).collect::<Vec<_>>().into(),
        dyn_flags: vec![true, false, true].into(),
        dyn_texts: vec![astr(""), astr("abc")].into(),
        dyn_leaves: vec![leaf(63, "deep"), Leaf::default(), leaf(-63, "z")].into(),
    }
}
fn enums_all() -> Enums {
    Enums { color: Color::Yellow, level: Level::High,
        colors: vec![Color::Red, Color::Green, Color::Blue, Color::Cyan, Color::Magenta, Color::Yellow].into() }
}
fn nested_deep() -> Nested {
    Nested { mid: Mid { id: 99.into(), leaf: leaf(-42, "level3"), extra: some(leaf(42, "extra")) }, after: true }
}
fn choice(v: OO_Choice_Value, tail: bool) -> Choice { Choice { value: v, tail } }

// ****************************************************************************
fn build() -> Gen {
    let mut g = Gen { lines: Vec::new(), names: HashSet::new(), valid: BTreeMap::new() };
    use DecodeError::*;

    // ************************************************************ valid
    g.section("valid: fixed size integers (spec 2.1-2.3)");
    g.valid("ints_default", "Ints", &Ints::default());
    g.valid("ints_max", "Ints", &ints_max());
    g.valid("ints_min", "Ints", &Ints {
        val_i1: (-1).into(), val_i7: (-127).into(), val_i16: (-i16::MAX).into(), val_i32: (-i32::MAX).into(),
        val_i33: (-((1_i64 << 33) - 1)).into(), val_i64: (-i64::MAX).into(), ..Default::default() });
    g.valid("ints_pow2", "Ints", &Ints {
        flag: false, val_u1: 1.into(), val_u7: 64.into(), val_u32: (1_u32 << 31).into(), val_u33: (1_u64 << 32).into(),
        val_u63: (1_u64 << 62).into(), val_u64: (1_u64 << 63).into(), val_i1: 0.into(), val_i7: (-64).into(),
        val_i16: (-16384).into(), val_i32: (1_i32 << 30).into(), val_i33: (-(1_i64 << 32)).into(),
        val_i64: (-(1_i64 << 62)).into() });
    g.valid("ints_small", "Ints", &Ints {
        flag: true, val_u1: 0.into(), val_u7: 1.into(), val_u32: 1.into(), val_u33: 1.into(), val_u63: 1.into(),
        val_u64: 1.into(), val_i1: (-1).into(), val_i7: (-1).into(), val_i16: 1.into(), val_i32: (-1).into(),
        val_i33: 1.into(), val_i64: (-1).into() });

    g.section("valid: dynamic integers (spec 2.4, 2.5)");
    g.valid("dynints_default", "DynInts", &DynInts::default());
    g.valid("dynints_max", "DynInts", &dynints_max());
    g.valid("dynints_min", "DynInts", &DynInts {
        val_u8d3: 1.into(), val_u32d4: 1.into(), val_u64d64: 1.into(), val_u64d7: 1.into(),
        val_i16d5: (-i16::MAX).into(), val_i64d8: (-i64::MAX).into() });
    g.valid("dynints_chunks", "DynInts", &DynInts {
        val_u8d3: 8.into(), val_u32d4: (1_u32 << 31).into(), val_u64d64: (1_u64 << 63).into(),
        val_u64d7: (1_u64 << 63).into(), val_i16d5: (-1).into(), val_i64d8: 256.into() });
    g.valid("dynints_chunk_edges", "DynInts", &DynInts {
        val_u8d3: 7.into(), val_u32d4: 16.into(), val_u64d64: 2.into(), val_u64d7: 127.into(),
        val_i16d5: 32.into(), val_i64d8: (-255).into() });

    g.section("valid: float / double (spec 2.6)");
    g.valid("floats_zero", "Floats", &Floats { val_f32: 0.0, val_f64: 0.0 });
    g.valid("floats_negzero", "Floats", &Floats { val_f32: -0.0, val_f64: -0.0 });
    g.valid("floats_inf", "Floats", &Floats { val_f32: f32::INFINITY, val_f64: f64::NEG_INFINITY });
    g.valid("floats_neginf", "Floats", &Floats { val_f32: f32::NEG_INFINITY, val_f64: f64::INFINITY });
    g.valid("floats_nan", "Floats", &Floats { val_f32: f32::from_bits(0x7fc0_0000),
        val_f64: f64::from_bits(0x7ff8_0000_0000_0000) });
    g.valid("floats_subnormal", "Floats", &Floats { val_f32: f32::from_bits(1), val_f64: f64::from_bits(1) });
    g.valid("floats_extremes", "Floats", &Floats { val_f32: f32::MAX, val_f64: -f64::MIN_POSITIVE });
    g.valid("floats_values", "Floats", &Floats { val_f32: -1.5, val_f64: std::f64::consts::PI });

    g.section("valid: fixed precision (spec 2.7); default is Overflow in Rust and C++");
    g.valid("fixedpoints_default", "FixedPoints", &FixedPoints::default());
    g.valid("fixedpoints_min", "FixedPoints", &FixedPoints { fp2: 0.0.into(), fp10: (-2.0).into(),
        lat: (-90.0).into(), lng: (-180.0).into(), fp53: (-1e6).into() });
    g.valid("fixedpoints_max", "FixedPoints", &FixedPoints { fp2: 1.0.into(), fp10: 3.0.into(),
        lat: 90.0.into(), lng: 180.0.into(), fp53: 1e6.into() });
    g.valid("fixedpoints_underflow", "FixedPoints", &FixedPoints { fp2: f64::NAN.into(), fp10: (-2.001).into(),
        lat: (-90.000001).into(), lng: f64::NEG_INFINITY.into(), fp53: (-1000000.001).into() });
    g.valid("fixedpoints_overflow", "FixedPoints", &FixedPoints { fp2: 1.5.into(), fp10: 3.001.into(),
        lat: 90.000001.into(), lng: f64::INFINITY.into(), fp53: 1000000.001.into() });
    g.valid("fixedpoints_zero", "FixedPoints", &FixedPoints { fp2: 0.5.into(), fp10: 0.0.into(),
        lat: 0.0.into(), lng: 0.0.into(), fp53: 0.0.into() });
    g.valid("fixedpoints_gnss", "FixedPoints", &fixedpoints_gnss());
    g.valid("fixedpoints_gnss_neg", "FixedPoints", &FixedPoints { fp2: 1.0.into(), fp10: 2.5.into(),
        lat: (-33.856784).into(), lng: (-151.215297).into(), fp53: (-0.5).into() });
    g.valid("fixedpoints_ties", "FixedPoints", &FixedPoints { fp2: 0.5.into(), fp10: TIE_FP10.into(),
        lat: TIE_LAT.into(), lng: TIE_LNG.into(), fp53: TIE_FP53.into() });

    g.section("valid: astr / binary (spec 2.8, 2.9)");
    g.valid("texts_empty", "Texts", &Texts::default());
    g.valid("texts_values", "Texts", &texts_values());
    g.valid("texts_ascii_edges", "Texts", &Texts { text: astr("\0 ~\x7f"), data: vec![0x80].into() });
    g.valid("texts_long", "Texts", &Texts {
        text: astr("The quick brown fox jumps over the lazy dog"),
        data: (0..300_u32).map(|v| (v % 256) as u8).collect::<Vec<u8>>().into() });

    g.section("valid: optional / repeated (spec 2.10-2.12)");
    g.valid("containers_default", "Containers", &Containers::default());
    g.valid("containers_full", "Containers", &containers_full());
    g.valid("containers_some_default", "Containers", &Containers {
        opt_u7: some(0.into()), opt_leaf: some(Leaf::default()), opt_text: some(astr("")),
        opt_color: some(Color::Red), dyn_ints: vec![0.into()].into(), dyn_flags: vec![false].into(),
        dyn_texts: vec![astr("")].into(), dyn_leaves: vec![Leaf::default()].into(), ..Default::default() });

    g.section("valid: enums (spec 2.13)");
    g.valid("enums_default", "Enums", &Enums::default());
    g.valid("enums_all", "Enums", &enums_all());
    g.valid("enums_first", "Enums", &Enums { color: Color::Red, level: Level::Mid,
        colors: vec![Color::Yellow, Color::Red].into() });

    g.section("valid: oneof (spec 2.14), one case per variant");
    g.valid("choice_default", "Choice", &Choice::default());
    g.valid("choice_flag", "Choice", &choice(OO_Choice_Value::Flag(true), true));
    g.valid("choice_num", "Choice", &choice(OO_Choice_Value::Num(-2.5), false));
    g.valid("choice_text", "Choice", &choice(OO_Choice_Value::Text(astr("oneof text")), true));
    g.valid("choice_leaf", "Choice", &choice(OO_Choice_Value::Leaf(leaf(-7, "leaf")), true));
    g.valid("choice_list", "Choice", &choice(OO_Choice_Value::List(IntList {
        items: vec![(-1).into(), 0.into(), 1.into(), 127.into(), (-127).into()].into() }), true));
    g.valid("choice_list_empty", "Choice", &choice(OO_Choice_Value::List(IntList::default()), true));
    g.valid("choice_nothing", "Choice", &choice(OO_Choice_Value::Nothing(Empty {}), true));
    g.valid("choice_big", "Choice", &choice(OO_Choice_Value::Big((-i32::MAX).into()), true));

    g.section("valid: nesting, empty message, everything");
    g.valid("nested_default", "Nested", &Nested::default());
    g.valid("nested_deep", "Nested", &nested_deep());
    g.valid("empty_msg", "Empty", &Empty {});
    g.valid("everything_default", "Everything", &Everything::default());
    g.valid("everything_full", "Everything", &Everything {
        ints: ints_max(), dyn_ints: dynints_max(),
        floats: Floats { val_f32: f32::from_bits(0x7fc0_0000), val_f64: -0.0 },
        fixed_points: fixedpoints_gnss(), texts: texts_values(), containers: containers_full(),
        enums: enums_all(), choice: choice(OO_Choice_Value::Text(astr("everything")), true),
        nested: nested_deep(), empty: Empty {}, tail: 5.into() });

    // ************************************************************ non-canonical
    g.section("noncanon: valid input that does not re-encode to itself");
    let ints_default = g.valid_bytes("ints_default");
    let ints_negzero = Bits::new().b(false).u(0, 1).u(0, 7).u(0, 32).u(0, 33).u(0, 63).u(0, 64)
        .sm(true, 0, 1).sm(true, 0, 7).sm(true, 0, 16).sm(true, 0, 32).sm(true, 0, 33).sm(true, 0, 64);
    g.noncanon("noncanon_ints_negzero", "Ints", &ints_negzero, &ints_default);

    let dyn_default = g.valid_bytes("dynints_default");
    let dyn_negzero = Bits::new().dyn_u(0, 8, 3).dyn_u(0, 32, 4).dyn_u(0, 64, 64).dyn_u(0, 64, 7)
        .dyn_i(true, 0, 16, 5).dyn_i(true, 0, 64, 8);
    g.noncanon("noncanon_dynints_negzero", "DynInts", &dyn_negzero, &dyn_default);
    let dyn_zero_chunks = Bits::new().dyn_chunks(&[0], 8, 3).dyn_chunks(&[0, 0], 32, 4).dyn_chunks(&[0], 64, 64)
        .dyn_chunks(&[0, 0, 0], 64, 7).b(true).dyn_chunks(&[0], 16, 5).b(false).dyn_chunks(&[0, 0], 64, 8);
    g.noncanon("noncanon_dynints_zero_chunks", "DynInts", &dyn_zero_chunks, &dyn_default);
    let dyn_nonminimal_val = DynInts { val_u8d3: 5.into(), val_u32d4: 5.into(), val_u64d7: 1.into(),
        val_i16d5: (-3).into(), val_i64d8: 1.into(), ..Default::default() };
    let dyn_nonminimal = Bits::new().dyn_chunks(&[5, 0], 8, 3).dyn_chunks(&[5, 0, 0], 32, 4).dyn_u(0, 64, 64)
        .dyn_chunks(&[1, 0], 64, 7).b(true).dyn_chunks(&[3, 0, 0, 0], 16, 5).b(false).dyn_chunks(&[1, 0, 0, 0, 0, 0, 0, 0], 64, 8);
    let canon = try_serialize(&dyn_nonminimal_val).unwrap().0;
    g.noncanon("noncanon_dynints_nonminimal", "DynInts", &dyn_nonminimal, &canon);

    let enums_canon = try_serialize(&Enums { color: Color::Red, level: Level::Low, colors: vec![Color::Blue].into() }).unwrap().0;
    let enums_nonminimal = Bits::new().dyn_chunks(&[0, 0], 32, 2).dyn_chunks(&[0], 32, 1)
        .dyn_chunks(&[1, 0, 0], 32, 3).dyn_chunks(&[2, 0], 32, 2);
    g.noncanon("noncanon_enums_nonminimal", "Enums", &enums_nonminimal, &enums_canon);

    let texts_canon = try_serialize(&Texts { text: astr("ab"), data: vec![1].into() }).unwrap().0;
    let texts_nonminimal = Bits::new().dyn_chunks(&[2, 0], 32, 4).u(b'a' as u64, 8).u(b'b' as u64, 8)
        .dyn_chunks(&[1, 0, 0], 32, 4).u(1, 8);
    g.noncanon("noncanon_texts_count_nonminimal", "Texts", &texts_nonminimal, &texts_canon);

    let choice_canon = try_serialize(&choice(OO_Choice_Value::Nothing(Empty {}), false)).unwrap().0;
    let choice_nonminimal = Bits::new().dyn_chunks(&[1, 1, 0], 32, 2).b(false);
    g.noncanon("noncanon_choice_index_nonminimal", "Choice", &choice_nonminimal, &choice_canon);

    // padding bits are not checked
    for case in ["choice_flag", "enums_all", "nested_deep"] {
        let (ty, bytes, bits) = g.valid[case].clone();
        assert!(bits % 8 != 0);
        let mut padded = bytes.clone();
        *padded.last_mut().unwrap() |= 0xff_u8 << (bits % 8);
        g.noncanon_bytes(&format!("noncanon_{case}_padding"), &ty, &padded, bits, &bytes);
    }

    // ************************************************************ invalid
    g.section("invalid: truncated input / empty input (UnexpectedEnd)");
    g.truncations("ints_max", &[0, 1, 44]);
    g.truncations("dynints_max", &[3, 32]);
    g.truncations("floats_nan", &[4, 11]);
    g.truncations("fixedpoints_gnss", &[10, 16]);
    g.truncations("texts_long", &[1, 30, 340]);
    g.truncations("containers_full", &[5, 37]);
    g.truncations("enums_all", &[1, 2]);
    g.truncations("choice_list", &[1, 3]);
    g.truncations("choice_num", &[8]);
    g.truncations("nested_deep", &[0, 4, 11]);
    let everything_len = g.valid_bytes("everything_full").len();
    g.truncations("everything_full", &[0, 1, everything_len / 2, everything_len - 1]);
    g.invalid("empty_input_choice", "Choice", UnexpectedEnd, false, &[]);
    g.invalid("empty_input_leaf", "Leaf", UnexpectedEnd, false, &[]);

    g.section("invalid: counts larger than the remaining input (UnexpectedEnd, must not allocate)");
    g.invalid_bits("huge_count_astr", "Texts", UnexpectedEnd, &Bits::new().dyn_u(u32::MAX as u64, 32, 4).u(0x41, 8));
    g.invalid_bits("huge_count_binary", "Texts", UnexpectedEnd,
        &Bits::new().dyn_u(0, 32, 4).dyn_u(u32::MAX as u64, 32, 4).u(0x41, 8).u(0x42, 8));
    g.invalid_bits("count_astr_one_short", "Texts", UnexpectedEnd,
        &Bits::new().dyn_u(3, 32, 4).u(0x41, 8).u(0x42, 8));
    g.invalid_bits("huge_count_repeated_enum", "Enums", UnexpectedEnd,
        &Bits::new().dyn_u(0, 32, 2).dyn_u(0, 32, 1).dyn_u(u32::MAX as u64, 32, 3).u(0, 16));
    g.invalid_bits("huge_count_repeated_msg", "IntList", UnexpectedEnd,
        &Bits::new().dyn_u(0x0fff_ffff, 32, 3).u(0, 64));
    // Containers with all optionals absent, fixed arrays default, then huge dyn_leaves count
    let containers_prefix = Bits::new().b(false).b(false).b(false).b(false)
        .sm(false, 0, 7).sm(false, 0, 7).sm(false, 0, 7)
        .sm(false, 0, 7).dyn_u(0, 32, 4).sm(false, 0, 7).dyn_u(0, 32, 4)
        .dyn_u(0, 32, 3).dyn_u(0, 32, 4).dyn_u(0, 32, 2);
    g.invalid_bits("huge_count_repeated_leaf", "Containers", UnexpectedEnd,
        &containers_prefix.clone().dyn_u(1 << 20, 32, 2).u(0, 64));
    // min_bits(Leaf) = 8 + 1 = 9: 4 elements need >= 36 bits, only 32 are left
    g.invalid_bits("count_repeated_leaf_min_bits", "Containers", UnexpectedEnd,
        &containers_prefix.clone().dyn_u(4, 32, 2).u(0, 32));
    g.invalid_bits("huge_count_repeated_bool", "Containers", UnexpectedEnd,
        &Bits::new().b(false).b(false).b(false).b(false).sm(false, 0, 7).sm(false, 0, 7).sm(false, 0, 7)
            .sm(false, 0, 7).dyn_u(0, 32, 4).sm(false, 0, 7).dyn_u(0, 32, 4)
            .dyn_u(0, 32, 3).dyn_u(u32::MAX as u64, 32, 4).u(0, 24));
    // count check comes before the content check
    g.invalid_bits("astr_non_ascii_truncated", "Texts", UnexpectedEnd,
        &Bits::new().dyn_u(5, 32, 4).u(0x41, 8).u(0x80, 8));

    g.section("invalid: unknown enum / oneof index (InvalidDiscriminant)");
    g.invalid_bits("enum_color_index_6", "Enums", InvalidDiscriminant,
        &Bits::new().dyn_u(6, 32, 2).dyn_u(0, 32, 1).dyn_u(0, 32, 3));
    g.invalid_bits("enum_color_index_max", "Enums", InvalidDiscriminant,
        &Bits::new().dyn_u(u32::MAX as u64, 32, 2).dyn_u(0, 32, 1).dyn_u(0, 32, 3));
    g.invalid_bits("enum_level_index_3", "Enums", InvalidDiscriminant,
        &Bits::new().dyn_u(0, 32, 2).dyn_u(3, 32, 1).dyn_u(0, 32, 3));
    g.invalid_bits("enum_in_array_index_7", "Enums", InvalidDiscriminant,
        &Bits::new().dyn_u(0, 32, 2).dyn_u(0, 32, 1).dyn_u(2, 32, 3).dyn_u(0, 32, 2).dyn_u(7, 32, 2));
    g.invalid_bits("enum_optional_index_6", "Containers", InvalidDiscriminant,
        &Bits::new().b(false).b(false).b(false).b(true).dyn_u(6, 32, 2).u(0, 64));
    // the error is detected before the (missing) rest of the message: first error wins
    g.invalid_bits("enum_invalid_then_end", "Enums", InvalidDiscriminant, &Bits::new().dyn_u(6, 32, 2));
    g.invalid_bits("oneof_index_7", "Choice", InvalidDiscriminant, &Bits::new().dyn_u(7, 32, 2).b(false));
    g.invalid_bits("oneof_index_max", "Choice", InvalidDiscriminant,
        &Bits::new().dyn_u(u32::MAX as u64, 32, 2).u(0, 8));
    g.invalid_bits("oneof_index_7_nested", "Everything", InvalidDiscriminant,
        &everything_prefix_to_choice().dyn_u(7, 32, 2).u(0, 64));

    g.section("invalid: values outside the valid range (InvalidValue)");
    let ints = |i16v: (bool, u64), i32v: (bool, u64), i64v: (bool, u64)| {
        Bits::new().b(false).u(0, 1).u(0, 7).u(0, 32).u(0, 33).u(0, 63).u(0, 64)
            .sm(false, 0, 1).sm(false, 0, 7).sm(i16v.0, i16v.1, 16).sm(i32v.0, i32v.1, 32)
            .sm(false, 0, 33).sm(i64v.0, i64v.1, 64)
    };
    g.invalid_bits("int16_neg_magnitude_8000", "Ints", InvalidValue, &ints((true, 0x8000), (false, 0), (false, 0)));
    g.invalid_bits("int16_pos_magnitude_ffff", "Ints", InvalidValue, &ints((false, 0xffff), (false, 0), (false, 0)));
    g.invalid_bits("int32_neg_magnitude_2p31", "Ints", InvalidValue, &ints((false, 0), (true, 1 << 31), (false, 0)));
    g.invalid_bits("int64_neg_magnitude_2p63", "Ints", InvalidValue, &ints((false, 0), (false, 0), (true, 1 << 63)));
    g.invalid_bits("int64_pos_magnitude_max", "Ints", InvalidValue, &ints((false, 0), (false, 0), (false, u64::MAX)));
    // first error wins: invalid i16 magnitude, input ends right after it
    let ints_cut = Bits::new().b(false).u(0, 1).u(0, 7).u(0, 32).u(0, 33).u(0, 63).u(0, 64)
        .sm(false, 0, 1).sm(false, 0, 7).sm(true, 0x8000, 16);
    g.invalid_bits("int16_invalid_then_end", "Ints", InvalidValue, &ints_cut);
    let dyn_ints = |i16v: (bool, u64), i64v: (bool, u64)| {
        Bits::new().dyn_u(0, 8, 3).dyn_u(0, 32, 4).dyn_u(0, 64, 64).dyn_u(0, 64, 7)
            .dyn_i(i16v.0, i16v.1, 16, 5).dyn_i(i64v.0, i64v.1, 64, 8)
    };
    g.invalid_bits("dyn_int16_neg_magnitude_8000", "DynInts", InvalidValue, &dyn_ints((true, 0x8000), (false, 0)));
    g.invalid_bits("dyn_int16_pos_magnitude_ffff", "DynInts", InvalidValue, &dyn_ints((false, 0xffff), (false, 0)));
    g.invalid_bits("dyn_int64_neg_magnitude_2p63", "DynInts", InvalidValue, &dyn_ints((false, 0), (true, 1 << 63)));
    g.invalid_bits("dyn_int64_pos_magnitude_max", "DynInts", InvalidValue, &dyn_ints((false, 0), (false, u64::MAX)));
    g.invalid_bits("oneof_dyn_int32_magnitude_2p31", "Choice", InvalidValue,
        &Bits::new().dyn_u(6, 32, 2).dyn_i(true, 1 << 31, 32, 7).b(false));
    g.invalid_bits("astr_non_ascii_80", "Texts", InvalidValue,
        &Bits::new().dyn_u(2, 32, 4).u(0x41, 8).u(0x80, 8).dyn_u(0, 32, 4));
    g.invalid_bits("astr_non_ascii_ff", "Texts", InvalidValue, &Bits::new().dyn_u(1, 32, 4).u(0xff, 8).dyn_u(0, 32, 4));
    g.invalid_bits("astr_non_ascii_nested", "Nested", InvalidValue,
        &Bits::new().u(1, 7).sm(false, 1, 7).dyn_u(2, 32, 4).u(0xc3, 8).u(0xa4, 8).b(false).b(true));
    g.invalid_bits("astr_non_ascii_in_array", "Containers", InvalidValue,
        &containers_prefix_to_dyn_texts().dyn_u(2, 32, 2).dyn_u(1, 32, 4).u(0x41, 8).dyn_u(1, 32, 4).u(0x9f, 8)
            .dyn_u(0, 32, 2));

    g.section("invalid: trailing data in exact mode (TrailingData); default mode accepts it");
    let b = g.valid_bytes("ints_default");
    g.invalid("trailing_ints_default", "Ints", TrailingData, true, &[b.as_slice(), &[0]].concat());
    g.invalid("trailing_empty_msg", "Empty", TrailingData, true, &[0]);
    let b = g.valid_bytes("choice_flag");
    g.invalid("trailing_choice_flag", "Choice", TrailingData, true, &[b.as_slice(), &[0xff, 0xff]].concat());
    let b = g.valid_bytes("everything_full");
    g.invalid("trailing_everything_full", "Everything", TrailingData, true, &[b.as_slice(), &[0x5a]].concat());

    g
}

/// Bits of a default `Everything` up to (not including) its `choice` attribute.
fn everything_prefix_to_choice() -> Bits {
    let e = Everything::default();
    let mut s = Biseri::new();
    e.ints.bit_serialize(&mut s); e.dyn_ints.bit_serialize(&mut s); e.floats.bit_serialize(&mut s);
    e.fixed_points.bit_serialize(&mut s); e.texts.bit_serialize(&mut s); e.containers.bit_serialize(&mut s);
    e.enums.bit_serialize(&mut s);
    let sizes = s.finish_add_data().unwrap();
    Bits::from_bytes(&s.get_data(), sizes.total_bits)
}
/// Bits of a default `Containers` up to (not including) `dyn_texts`.
fn containers_prefix_to_dyn_texts() -> Bits {
    Bits::new().b(false).b(false).b(false).b(false)
        .sm(false, 0, 7).sm(false, 0, 7).sm(false, 0, 7)
        .sm(false, 0, 7).dyn_u(0, 32, 4).sm(false, 0, 7).dyn_u(0, 32, 4)
        .dyn_u(0, 32, 3).dyn_u(0, 32, 4)
}

fn render(g: &Gen) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "# bitis golden vectors -- GENERATED by golden/gen_rust, do not edit (see golden/README.md)");
    let _ = writeln!(s, "# schema: golden/schema/golden.bitis, schema hash {:016x}", golden::BITIS_SCHEMA_HASH);
    let _ = writeln!(s, "# valid    <case> <MsgType> <total_bits> <hex>");
    let _ = writeln!(s, "# noncanon <case> <MsgType> <total_bits> <input_hex> <canonical_hex>");
    let _ = writeln!(s, "# invalid  <case> <MsgType> <error> <default|exact> <hex>");
    let _ = writeln!(s, "# hex: lower-case, '-' = empty input");
    for l in &g.lines { let _ = writeln!(s, "{l}"); }
    s
}

fn main() {
    let check = std::env::args().any(|a| a == "--check");
    let g = build();
    let content = render(&g);
    let n = |p: &str| g.lines.iter().filter(|l| l.starts_with(p)).count();
    if check {
        let committed = std::fs::read_to_string(MANIFEST_PATH).unwrap_or_default();
        if committed != content {
            eprintln!("golden/vectors/manifest.txt is out of date; regenerate it with\n  \
                cargo run --manifest-path golden/gen_rust/Cargo.toml");
            std::process::exit(1);
        }
        println!("manifest up to date ({} valid, {} noncanon, {} invalid)", n("valid "), n("noncanon "), n("invalid "));
    } else {
        std::fs::write(MANIFEST_PATH, content).expect("write manifest");
        println!("wrote {} ({} valid, {} noncanon, {} invalid)", MANIFEST_PATH, n("valid "), n("noncanon "), n("invalid "));
    }
}
