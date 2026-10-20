#![allow(dead_code, non_snake_case, nonstandard_style)]

use bitis_lib::*;
use strum_macros::AsRefStr;

/// Version of the bitis compiler that generated this file.
pub const BITIS_VERSION: &str = "{BITIS_VERSION}";
/// Schema hash: first 8 bytes of SHA-256 over the canonical schema (see bitis docs/WIRE_FORMAT.md).
pub const BITIS_SCHEMA_HASH: u64 = 0x18fe2976a5287710;

// Enums
/// Enum doc with special chars: <html> & "quotes" \ 'single'
/// second line
#[derive(BiserdiEnum, Debug, Clone, PartialEq, Copy, Default, AsRefStr)]
#[biserdi_enum_id_dynbits(2)]
#[allow(nonstandard_style)]
pub enum Kind {
  /// first value
  #[default] KindA,
  /// second value
  KindB,
}


// Enums for oneof
#[derive(BiserdiOneOf, Debug, Clone, PartialEq, AsRefStr)]
#[biserdi_enum_id_dynbits(2)]
#[allow(nonstandard_style)]
pub enum OO_Documented_Sel {
  /// element doc
  Flag(bool),
  /// second element doc
  Num(IntWithGivenBitSize<u16, 3>),
}
impl Default for OO_Documented_Sel {
  fn default() -> Self { Self::Flag(call_default()) }
}


// Messages
/// Message doc
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct Documented {
  /// attribute doc
  pub value: IntWithGivenBitSize<u16, 4>,
  /// multi
  /// line attribute doc
  pub kind: Kind,
  /// oneof doc
  pub sel: OO_Documented_Sel,
}


