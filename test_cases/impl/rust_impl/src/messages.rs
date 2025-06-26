use bitis_lib::*;

// Enums


// Enums for oneof


// Messages
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct MsgSimpleBaseOneInt {
  pub param_1: IntWithGivenBitSize<u16, 11>,
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct MsgSimpleBaseThreeInt {
  pub param_1: IntWithGivenBitSize<u16, 11>,
  pub param_2: IntWithGivenBitSize<u16, 6>,
  pub param_3: IntWithGivenBitSize<u16, 11>,
  pub param_4: DynInteger<u16, 6, 4>,
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct MsgSimpleTestBase {
  pub param_1: IntWithGivenBitSize<u16, 11>,
  pub param_2: bool,
  pub param_3: IntWithGivenBitSize<i16, 5>,
  pub name: BitisAString<4>,
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct MsgSimpleTestFP {
  pub param_1: bool,
  pub fp: FixPrecisionMinMax<10, -1, 1>,
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct MsgSimpleOpt {
  pub param_1: IntWithGivenBitSize<u16, 11>,
  pub param_2: BitisOption<bool>,
  pub param_3: BitisOption<IntWithGivenBitSize<u16, 11>>,
  pub param_4: BitisOption<FixPrecisionMinMax<10, -1, 1>>,
}
