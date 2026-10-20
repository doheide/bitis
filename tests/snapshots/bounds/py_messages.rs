#![allow(dead_code, non_snake_case, nonstandard_style)]

use bitis_lib::*;
use strum_macros::AsRefStr;

/// Version of the bitis compiler that generated this file.
pub const BITIS_VERSION: &str = "{BITIS_VERSION}";
/// Schema hash: first 8 bytes of SHA-256 over the canonical schema (see bitis docs/WIRE_FORMAT.md).
pub const BITIS_SCHEMA_HASH: u64 = 0x940ea6464ca7ba60;

// Enums
#[derive(BiserdiEnum, Debug, Clone, PartialEq, Copy, Default, AsRefStr)]
#[biserdi_enum_id_dynbits(2)]
#[allow(nonstandard_style)]
pub enum Kind {
  #[default] Small,
  Large,
}


// Enums for oneof
#[derive(BiserdiOneOf, Debug, Clone, PartialEq, AsRefStr)]
#[biserdi_enum_id_dynbits(2)]
#[allow(nonstandard_style)]
pub enum OO_Bounded_Payload {
  Text(BoundedAString<2, 5>),
  Raw(BoundedAString<4, 6>),
  P(Point),
}
impl Default for OO_Bounded_Payload {
  fn default() -> Self { Self::Text(call_default()) }
}


// Messages
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct Point {
  pub x: IntWithGivenBitSize<u16, 8>,
  pub y: IntWithGivenBitSize<u16, 8>,
}

/// Bounded strings, bytes and lists
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct Bounded {
  pub name: BoundedAString<4, 16>,
  pub blob: BoundedBinary<3, 8>,
  pub tags: BoundedDynArray<BoundedAString<4, 8>,2,4>,
  pub note: BitisOption<BoundedAString<4, 32>>,
  pub points: BoundedDynArray<Point,3,3>,
  pub kinds: BoundedDynArray<Kind,1,2>,
  pub pair: FixedArray<BoundedBinary<2, 3>,2>,
  pub unbounded: BitisAString<4>,
  pub payload: OO_Bounded_Payload,
}


