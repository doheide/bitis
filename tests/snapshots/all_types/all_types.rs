#![allow(dead_code, non_snake_case, nonstandard_style)]

use bitis_lib::*;
use strum_macros::AsRefStr;

/// Version of the bitis compiler that generated this file.
pub const BITIS_VERSION: &str = "{BITIS_VERSION}";
/// Schema hash: first 8 bytes of SHA-256 over the canonical schema (see bitis docs/WIRE_FORMAT.md).
pub const BITIS_SCHEMA_HASH: u64 = 0xc10a57d77c346314;

// Enums
#[derive(BiserdiEnum, Debug, Clone, PartialEq, Copy, Default, AsRefStr)]
#[biserdi_enum_id_dynbits(2)]
#[allow(nonstandard_style)]
pub enum Color {
  Red,
  #[default] Green,
  BlueLight,
}


// Enums for oneof
#[derive(BiserdiOneOf, Debug, Clone, PartialEq, AsRefStr)]
#[biserdi_enum_id_dynbits(3)]
#[allow(nonstandard_style)]
pub enum OO_AllTypes_Choice {
  Flag(bool),
  Number(IntWithGivenBitSize<u16, 12>),
  InnerVal(Inner),
  ColorVal(Color),
  Text(BitisAString<3>),
  Nothing(Empty),
  Raw(Binary<4>),
}
impl Default for OO_AllTypes_Choice {
  fn default() -> Self { Self::Flag(call_default()) }
}


// Messages
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct Inner {
  pub a: IntWithGivenBitSize<u16, 3>,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct Empty {
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct AllTypes {
  pub b: bool,
  pub u1: IntWithGivenBitSize<u16, 1>,
  pub u17: IntWithGivenBitSize<u32, 17>,
  pub u64v: IntWithGivenBitSize<u64, 64>,
  pub i5: IntWithGivenBitSize<i16, 5>,
  pub i33: IntWithGivenBitSize<i64, 33>,
  pub ud: DynInteger<u32, 32, 4>,
  pub id: DynInteger<i16, 16, 3>,
  pub f: f32,
  pub d: f64,
  pub fp: FixPrecisionMinMax<10, -2, 3>,
  pub lat: FixPrecisionMinMax<34, -90, 90>,
  pub s: BitisAString<4>,
  pub data: Binary<6>,
  pub color: Color,
  pub inner: Inner,
  pub empty: Empty,
  pub opt_u: BitisOption<IntWithGivenBitSize<u16, 7>>,
  pub opt_inner: BitisOption<Inner>,
  pub opt_empty: BitisOption<Empty>,
  pub rf_b: FixedArray<bool,3>,
  pub rf_inner: FixedArray<Inner,2>,
  pub rd_u: DynArray<IntWithGivenBitSize<u16, 4>,2>,
  pub rd_color: DynArray<Color,3>,
  pub rd_inner: DynArray<Inner,4>,
  pub choice: OO_AllTypes_Choice,
}


