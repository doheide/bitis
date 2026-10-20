#![allow(dead_code, non_snake_case, nonstandard_style)]

use bitis_lib::*;
use strum_macros::AsRefStr;

/// Version of the bitis compiler that generated this file.
pub const BITIS_VERSION: &str = "0.11.0";
/// Schema hash: first 8 bytes of SHA-256 over the canonical schema (see bitis docs/WIRE_FORMAT.md).
pub const BITIS_SCHEMA_HASH: u64 = 0xb637cfeeccc2d168;

// Enums
#[derive(BiserdiEnum, Debug, Clone, PartialEq, Copy, Default, AsRefStr)]
#[biserdi_enum_id_dynbits(2)]
#[allow(nonstandard_style)]
pub enum Color {
  Red,
  #[default] Green,
  Blue,
  Cyan,
  Magenta,
  Yellow,
}
#[derive(BiserdiEnum, Debug, Clone, PartialEq, Copy, Default, AsRefStr)]
#[biserdi_enum_id_dynbits(1)]
#[allow(nonstandard_style)]
pub enum Level {
  #[default] Low,
  Mid,
  High,
}


// Enums for oneof
#[derive(BiserdiOneOf, Debug, Clone, PartialEq, AsRefStr)]
#[biserdi_enum_id_dynbits(2)]
#[allow(nonstandard_style)]
pub enum OO_Choice_Value {
  Flag(bool),
  Num(f64),
  Text(BitisAString<4>),
  Leaf(Leaf),
  List(IntList),
  Nothing(Empty),
  Big(DynInteger<i32, 32, 7>),
}
impl Default for OO_Choice_Value {
  fn default() -> Self { Self::Flag(call_default()) }
}


// Messages
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct Empty {
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct Ints {
  pub flag: bool,
  pub val_u1: IntWithGivenBitSize<u16, 1>,
  pub val_u7: IntWithGivenBitSize<u16, 7>,
  pub val_u32: IntWithGivenBitSize<u32, 32>,
  pub val_u33: IntWithGivenBitSize<u64, 33>,
  pub val_u63: IntWithGivenBitSize<u64, 63>,
  pub val_u64: IntWithGivenBitSize<u64, 64>,
  pub val_i1: IntWithGivenBitSize<i16, 1>,
  pub val_i7: IntWithGivenBitSize<i16, 7>,
  pub val_i16: IntWithGivenBitSize<i16, 16>,
  pub val_i32: IntWithGivenBitSize<i32, 32>,
  pub val_i33: IntWithGivenBitSize<i64, 33>,
  pub val_i64: IntWithGivenBitSize<i64, 64>,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct DynInts {
  pub val_u8d3: DynInteger<u16, 8, 3>,
  pub val_u32d4: DynInteger<u32, 32, 4>,
  pub val_u64d64: DynInteger<u64, 64, 64>,
  pub val_u64d7: DynInteger<u64, 64, 7>,
  pub val_i16d5: DynInteger<i16, 16, 5>,
  pub val_i64d8: DynInteger<i64, 64, 8>,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct Floats {
  pub val_f32: f32,
  pub val_f64: f64,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct FixedPoints {
  pub fp2: FixPrecisionMinMax<2, 0, 1>,
  pub fp10: FixPrecisionMinMax<10, -2, 3>,
  pub lat: FixPrecisionMinMax<34, -90, 90>,
  pub lng: FixPrecisionMinMax<35, -180, 180>,
  pub fp53: FixPrecisionMinMax<53, -1000000, 1000000>,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct Texts {
  pub text: BitisAString<4>,
  pub data: Binary<4>,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct Leaf {
  pub val: IntWithGivenBitSize<i16, 7>,
  pub tag: BitisAString<4>,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct Mid {
  pub id: IntWithGivenBitSize<u16, 7>,
  pub leaf: Leaf,
  pub extra: BitisOption<Leaf>,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct Nested {
  pub mid: Mid,
  pub after: bool,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct Containers {
  pub opt_u7: BitisOption<IntWithGivenBitSize<u16, 7>>,
  pub opt_leaf: BitisOption<Leaf>,
  pub opt_text: BitisOption<BitisAString<4>>,
  pub opt_color: BitisOption<Color>,
  pub fixed_ints: FixedArray<IntWithGivenBitSize<i16, 7>,3>,
  pub fixed_leaves: FixedArray<Leaf,2>,
  pub dyn_ints: DynArray<IntWithGivenBitSize<u16, 7>,3>,
  pub dyn_flags: DynArray<bool,4>,
  pub dyn_texts: DynArray<BitisAString<4>,2>,
  pub dyn_leaves: DynArray<Leaf,2>,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct Enums {
  pub color: Color,
  pub level: Level,
  pub colors: DynArray<Color,3>,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct IntList {
  pub items: DynArray<IntWithGivenBitSize<i16, 7>,3>,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct Choice {
  pub value: OO_Choice_Value,
  pub tail: bool,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct BoundsItem {
  pub val: IntWithGivenBitSize<u16, 4>,
  pub label: BitisOption<BoundedAString<2, 3>>,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct Bounds {
  pub name: BoundedAString<4, 8>,
  pub blob: BoundedBinary<3, 4>,
  pub tags: BoundedDynArray<BoundedAString<2, 2>,2,3>,
  pub note: BitisOption<BoundedAString<4, 5>>,
  pub items: BoundedDynArray<BoundsItem,3,2>,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct Everything {
  pub ints: Ints,
  pub dyn_ints: DynInts,
  pub floats: Floats,
  pub fixed_points: FixedPoints,
  pub texts: Texts,
  pub containers: Containers,
  pub enums: Enums,
  pub choice: Choice,
  pub nested: Nested,
  pub empty: Empty,
  pub tail: IntWithGivenBitSize<u16, 3>,
}


