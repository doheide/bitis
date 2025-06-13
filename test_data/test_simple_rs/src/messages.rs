use bitis_lib::*;

// Enums
#[derive(BiserdiEnum, Debug, Clone, PartialEq, Copy, Default)]
#[biserdi_enum_id_dynbits(4)]
#[allow(nonstandard_style)]
pub enum Numbers {
  One,
  #[default] Two,
  Three,
  Four,
}
impl std::fmt::Display for Numbers {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self) }
}


// Enums for oneof
#[derive(BiserdiOneOf, Debug, Clone, PartialEq)]
#[biserdi_enum_id_dynbits(4)]
#[allow(nonstandard_style)]
pub enum OO_ParamTestWithInner_Action {
  Inner(Inner),
  Val(VarWithGivenBitSize<u8, 3>),
}
impl std::fmt::Display for OO_ParamTestWithInner_Action {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self) }
}
impl Default for OO_ParamTestWithInner_Action {
  fn default() -> Self { Self::Val(VarWithGivenBitSize<u8, 3>::default()) }
}


// Messages
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct Inner {
  pub name: BitisString<4>,
  pub val: VarWithGivenBitSize<u8, 3>,
  pub num: Numbers,
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct ParamTestWithInner {
  pub name: BitisString<4>,
  pub param_1: DynInteger<u8, 4>,
  pub param_2: bool,
  pub action: OO_ParamTestWithInner_Action,
  pub fp_val: FixPrecisionMinMax<10, 1, 2>,
  pub val_opt: BitisOption<VarWithGivenBitSize<u8, 3>>,
  pub val_fixed_array: FixedArray<VarWithGivenBitSize<u8, 3>,6>,
}
