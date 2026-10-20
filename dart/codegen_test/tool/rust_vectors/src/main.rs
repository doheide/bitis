//! Prints the vectors (see Cargo.toml) as JSON to stdout:
//! `{"generator": ..., "schemas": {name: schema hash}, "vectors": [{schema, name, message, value, bits, hex}]}`
//! where `value` is the JSON representation of the encoded value (`null`: the default value).
use bitis_lib::*;
use serde_json::{json, Value as J};

/// Converts a JSON value (JSON mapping of the Dart target) into a Rust value.
pub trait FromJ { fn from_j(j: &J) -> Self; }

/// Lower case without underscores (see build.rs).
pub fn norm(s: &str) -> String { s.chars().filter(|c| *c != '_').flat_map(|c| c.to_lowercase()).collect() }
/// The attribute `key` of a message (`null` if missing).
pub fn field<'a>(j: &'a J, key: &str) -> &'a J {
    static NULL: J = J::Null;
    j.as_object().expect("message must be a JSON object").get(key).unwrap_or(&NULL)
}
/// `{"element": value}` of a oneof.
pub fn oneof_entry(j: &J) -> (&str, &J) {
    let o = j.as_object().expect("oneof must be a JSON object");
    assert_eq!(o.len(), 1, "oneof must have exactly one key: {j}");
    let (k, v) = o.iter().next().unwrap();
    (k.as_str(), v)
}

/// Integer from a JSON number or a decimal string.
fn int(j: &J) -> i128 {
    match j {
        J::Number(n) => n.as_i64().map(|v| v as i128).or(n.as_u64().map(|v| v as i128)).expect("integer"),
        J::String(s) => s.parse().expect("decimal integer string"),
        _ => panic!("expected an integer, got {j}"),
    }
}
fn double(j: &J) -> f64 {
    match j {
        J::Number(n) => n.as_f64().unwrap(),
        J::String(s) if s == "NaN" => f64::NAN,
        J::String(s) if s == "Infinity" => f64::INFINITY,
        J::String(s) if s == "-Infinity" => f64::NEG_INFINITY,
        _ => panic!("expected a number, got {j}"),
    }
}
fn base64(s: &str) -> Vec<u8> {
    let val = |c: u8| -> u32 {
        match c {
            b'A'..=b'Z' => (c - b'A') as u32, b'a'..=b'z' => (c - b'a' + 26) as u32,
            b'0'..=b'9' => (c - b'0' + 52) as u32, b'+' | b'-' => 62, b'/' | b'_' => 63,
            _ => panic!("invalid base64 character"),
        }
    };
    let (mut out, mut acc, mut n) = (vec![], 0u32, 0);
    for c in s.bytes().filter(|c| *c != b'=') {
        acc = (acc << 6) | val(c);
        n += 6;
        if n >= 8 { n -= 8; out.push((acc >> n) as u8); acc &= (1 << n) - 1; }
    }
    out
}

impl FromJ for bool { fn from_j(j: &J) -> Self { j.as_bool().expect("bool") } }
impl FromJ for f32 { fn from_j(j: &J) -> Self { double(j) as f32 } }
impl FromJ for f64 { fn from_j(j: &J) -> Self { double(j) } }
macro_rules! int_from_j {
    ($($t:ty),*) => {$(
        impl<const N: u64> FromJ for IntWithGivenBitSize<$t, N> {
            fn from_j(j: &J) -> Self { Self { val: <$t>::try_from(int(j)).expect("integer out of range") } }
        }
        impl<const X: u8, const Y: u8> FromJ for DynInteger<$t, X, Y> {
            fn from_j(j: &J) -> Self { Self { val: <$t>::try_from(int(j)).expect("integer out of range") } }
        }
    )*};
}
int_from_j!(u16, u32, u64, i16, i32, i64);
impl<const N: u8, const MIN: i64, const MAX: i64> FromJ for FixPrecisionMinMax<N, MIN, MAX> {
    fn from_j(j: &J) -> Self {
        let val = match j.get("state").and_then(|s| s.as_str()) {
            Some("underflow") => FixPrecisionVal::Underflow,
            Some("overflow") => FixPrecisionVal::Overflow,
            _ => FixPrecisionVal::Value(double(j)),
        };
        Self { val }
    }
}
impl<const Y: u8> FromJ for BitisAString<Y> {
    fn from_j(j: &J) -> Self { Self::try_from(j.as_str().expect("string")).expect("ASCII string") }
}
impl<const Y: u8> FromJ for Binary<Y> {
    fn from_j(j: &J) -> Self { Self { val: base64(j.as_str().expect("base64 string")) } }
}
impl<T: FromJ + BiserdiTrait + Default> FromJ for BitisOption<T> {
    fn from_j(j: &J) -> Self { Self { val: if j.is_null() { None } else { Some(T::from_j(j)) } } }
}
impl<T: FromJ + BiserdiTrait + Default + std::fmt::Debug, const N: usize> FromJ for FixedArray<T, N> {
    fn from_j(j: &J) -> Self {
        let a = j.as_array().expect("array");
        assert_eq!(a.len(), N, "fixed array length");
        Self { val: std::array::from_fn(|i| T::from_j(&a[i])) }
    }
}
impl<T: FromJ + BiserdiTrait + Default, const Y: u8> FromJ for DynArray<T, Y> {
    fn from_j(j: &J) -> Self { Self { val: j.as_array().expect("array").iter().map(T::from_j).collect() } }
}

/// Bytes and number of bits.
pub type Encoded = (Vec<u8>, u64);

/// Encodes with the Rust runtime and checks that decoding gives the same bytes again.
pub fn encode_as<T: FromJ + BiserdiTrait + Default>(j: Option<&J>) -> Encoded {
    let v = match j { Some(j) => T::from_j(j), None => T::default() };
    let (bytes, sizes) = try_serialize(&v).expect("value can not be encoded");
    let back = try_deserialize_exact::<T>(&bytes).expect("encoded value can not be decoded");
    assert_eq!(try_serialize(&back).unwrap().0, bytes, "re-encoding differs");
    (bytes, sizes.total_bits)
}

#[allow(dead_code, non_camel_case_types, non_snake_case)]
#[path = "gen/cara.rs"] mod cara_gen;
#[allow(dead_code, non_camel_case_types, non_snake_case)]
#[path = "gen/test_schema.rs"] mod test_schema_gen;
#[allow(dead_code, non_camel_case_types, non_snake_case)]
#[path = "gen/all_types.rs"] mod all_types_gen;

mod cara { use super::*; pub use super::cara_gen::*; include!(concat!(env!("OUT_DIR"), "/glue_cara.rs")); }
mod test_schema { use super::*; pub use super::test_schema_gen::*; include!(concat!(env!("OUT_DIR"), "/glue_test_schema.rs")); }
mod all_types { use super::*; pub use super::all_types_gen::*; include!(concat!(env!("OUT_DIR"), "/glue_all_types.rs")); }

type EncodeFn = fn(&str, Option<&J>) -> Option<Encoded>;

fn main() {
    let path = std::env::args().nth(1).expect("usage: bitis_dart_vectors <vector_inputs.json>");
    let inputs: J = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let schemas: [(&str, EncodeFn, &[&str], u64); 3] = [
        ("cara", cara::encode, cara::MESSAGES, cara::BITIS_SCHEMA_HASH),
        ("test_schema", test_schema::encode, test_schema::MESSAGES, test_schema::BITIS_SCHEMA_HASH),
        ("all_types", all_types::encode, all_types::MESSAGES, all_types::BITIS_SCHEMA_HASH),
    ];
    let hex = |b: &[u8]| b.iter().map(|x| format!("{x:02x}")).collect::<String>();
    let mut vectors = vec![];
    for (schema, encode, messages, _) in &schemas {
        for m in *messages {
            let (bytes, bits) = encode(m, None).unwrap();
            vectors.push(json!({"schema": schema, "name": format!("default_{m}"), "message": m, "value": null,
                                "bits": bits, "hex": hex(&bytes)}));
        }
    }
    for v in inputs["vectors"].as_array().expect("vectors") {
        let schema = v["schema"].as_str().unwrap();
        let (_, encode, _, _) = schemas.iter().find(|s| s.0 == schema).expect("unknown schema");
        let msg = v["message"].as_str().unwrap();
        let (bytes, bits) = encode(msg, Some(&v["value"])).unwrap_or_else(|| panic!("unknown message {msg}"));
        vectors.push(json!({"schema": schema, "name": v["name"], "message": msg, "value": v["value"],
                            "bits": bits, "hex": hex(&bytes)}));
    }
    let hashes: serde_json::Map<String, J> = schemas.iter().map(|s| (s.0.to_string(), json!(format!("{:016x}", s.3)))).collect();
    let out = json!({
        "generator": format!("bitis {} Rust runtime (dart/codegen_test/tool/rust_vectors)", BITIS_COMPILER_VERSION),
        "schemas": hashes,
        "vectors": vectors,
    });
    println!("{}", serde_json::to_string_pretty(&out).unwrap());
}
