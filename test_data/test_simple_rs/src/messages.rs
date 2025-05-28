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
  pub param_1: DynInteger<u8, 4>,
  pub param_2: bool,
  pub action: OO_ParamTestWithInner_Action,
  pub fp_val: FixPrecisionMinMax<10, 1, 2>,
  pub val_opt: Option<VarWithGivenBitSize<u8, 3>>,
  pub val_fixed_array: FixedArray<VarWithGivenBitSize<u8, 3>,6>,
}
