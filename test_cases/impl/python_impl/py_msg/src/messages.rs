#![allow(dead_code, non_snake_case, nonstandard_style)]

use bitis_lib::*;
use strum_macros::AsRefStr;

// Enums


// Enums for oneof
#[derive(BiserdiOneOf, Debug, Clone, PartialEq, AsRefStr)]
#[biserdi_enum_id_dynbits(4)]
#[allow(nonstandard_style)]
pub enum OO_MsgOoSimpleBase_Value {
  Int(IntWithGivenBitSize<u16, 8>),
  Number(f64),
  TrueFalse(bool),
}
impl Default for OO_MsgOoSimpleBase_Value {
//  fn default() -> Self { Self::TrueFalse(bool(call_default()) }
  fn default() -> Self { Self::TrueFalse(call_default()) }
}
#[derive(BiserdiOneOf, Debug, Clone, PartialEq, AsRefStr)]
#[biserdi_enum_id_dynbits(4)]
#[allow(nonstandard_style)]
pub enum OO_MsgOoNestedBase_Value {
  Inner(MsgSimpleBaseOneInt),
  Number(f64),
  TrueFalse(bool),
}
impl Default for OO_MsgOoNestedBase_Value {
//  fn default() -> Self { Self::Inner(MsgSimpleBaseOneInt(call_default()) }
  fn default() -> Self { Self::Inner(call_default()) }
}


// Messages
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct MsgOOSimpleBase {
  pub id: IntWithGivenBitSize<u16, 8>,
  pub value: OO_MsgOoSimpleBase_Value,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct MsgSimpleBaseOneInt {
  pub param_1: IntWithGivenBitSize<u16, 11>,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct MsgOONestedBase {
  pub id: IntWithGivenBitSize<u16, 8>,
  pub value: OO_MsgOoNestedBase_Value,
}


