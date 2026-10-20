#![allow(dead_code, non_snake_case, nonstandard_style)]

use bitis_lib::*;
use strum_macros::AsRefStr;

/// Version of the bitis compiler that generated this file.
pub const BITIS_VERSION: &str = "{BITIS_VERSION}";
/// Schema hash: first 8 bytes of SHA-256 over the canonical schema (see bitis docs/WIRE_FORMAT.md).
pub const BITIS_SCHEMA_HASH: u64 = 0xf9d3f02b9ff943e5;

// Enums
#[derive(BiserdiEnum, Debug, Clone, PartialEq, Copy, Default, AsRefStr)]
#[biserdi_enum_id_dynbits(1)]
#[allow(nonstandard_style)]
pub enum E {
  #[default] A,
  B,
}


// Enums for oneof
#[derive(BiserdiOneOf, Debug, Clone, PartialEq, AsRefStr)]
#[biserdi_enum_id_dynbits(2)]
#[allow(nonstandard_style)]
pub enum OO_Top_Outer {
  Leaf(Leaf),
  Inner(OO_Outer_Inner),
}
impl Default for OO_Top_Outer {
  fn default() -> Self { Self::Leaf(call_default()) }
}
#[derive(BiserdiOneOf, Debug, Clone, PartialEq, AsRefStr)]
#[biserdi_enum_id_dynbits(1)]
#[allow(nonstandard_style)]
pub enum OO_Outer_Inner {
  X(bool),
  E(E),
}
impl Default for OO_Outer_Inner {
  fn default() -> Self { Self::X(call_default()) }
}


// Messages
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct Top {
  pub m: Middle,
  pub outer: OO_Top_Outer,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct Middle {
  pub leaves: DynArray<Leaf,2>,
  pub e: BitisOption<E>,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct Leaf {
  pub v: IntWithGivenBitSize<u16, 2>,
}


