use bitis_lib::*;

// Enums


// Enums for oneof


// Messages
#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct MsgSimpleTest {
  pub param_1: VarWithGivenBitSize<u8, 4>,
  pub param_2: bool,
  pub param_3: VarWithGivenBitSize<i8, 5>,
}
