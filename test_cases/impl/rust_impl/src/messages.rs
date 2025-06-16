use bitis_lib::*;

// Enums


// Enums for oneof


// Messages
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct MsgWithDynInt {
  pub val: DynInteger<u8, 4, 3>,
  pub signed_val: DynInteger<i8, 4, 3>,
}
