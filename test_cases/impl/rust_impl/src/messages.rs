use bitis_lib::*;

// Enums


// Enums for oneof


// Messages
#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct MsgSimpleBaseOneInt {
  pub param_1: VarWithGivenBitSize<u16, 11>,
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct MsgSimpleBaseThreeInt {
  pub param_1: VarWithGivenBitSize<u16, 11>,
  pub param_2: VarWithGivenBitSize<u8, 6>,
  pub param_3: VarWithGivenBitSize<u16, 11>,
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct MsgSimpleTestBase {
  pub param_1: VarWithGivenBitSize<u16, 11>,
  pub param_2: bool,
  pub param_3: VarWithGivenBitSize<i8, 5>,
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct MsgSimpleTestFp {
  pub param_1: bool,
  pub fp: FixPrecisionMinMax<10, -1, 1>,
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct MsgSimpleOpt {
  pub param_1: VarWithGivenBitSize<u16, 11>,
  pub param_2: BitisOption<bool>,
  pub param_3: BitisOption<VarWithGivenBitSize<u16, 11>>,
  pub param_4: BitisOption<FixPrecisionMinMax<10, -1, 1>>,
}
