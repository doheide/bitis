use bitis_lib::*;

// Enums

#[derive(BiserdiEnum, Debug, Clone, PartialEq)]
#[biserdi_enum_id_dynbits(4)]
#[allow(nonstandard_style)]
pub enum Numbers {
  One,
  Two,
  Three,
  Four,
}

// Enums for oneof

#[derive(BiserdiOneOf, Debug, Clone, PartialEq)]
#[biserdi_enum_id_dynbits(4)]
#[allow(nonstandard_style)]
pub enum OO_ParamTestWithInner_Action {
  Inner(Inner),
  Val(VarWithGivenBitSize<u8, 3>),
}

// Messages

#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct Inner {
  pub val: VarWithGivenBitSize<u8, 3>,
  pub num: Numbers,
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct ParamTestWithInner {
  pub param_1: VarWithGivenBitSize<u8, 4>,
  pub param_2: bool,
  pub action: OO_ParamTestWithInner_Action,
}
