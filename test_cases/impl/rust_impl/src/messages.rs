use bitis_lib::*;

// Enums


// Enums for oneof
#[derive(BiserdiOneOf, Debug, Clone, PartialEq)]
#[biserdi_enum_id_dynbits(2)]
#[allow(nonstandard_style)]
pub enum OO_MsgKvoo_Value {
  StrVal(BitisString<4>),
  NumVal(f64),
  BoolVal(bool),
  IntVal(DynInteger<i32, 7>),
}
impl std::fmt::Display for OO_MsgKvoo_Value {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self) }
}
impl Default for OO_MsgKvoo_Value {
  fn default() -> Self { Self::BoolVal(bool::default()) }
}


// Messages
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct MsgKVSimple {
  pub key: BitisString<4>,
  pub value: BitisString<4>,
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct MsgKVMapSimple {
  pub entries: DynArray<MsgKVSimple,4>,
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct MsgKVOO {
  pub key: BitisString<4>,
  pub value: OO_MsgKvoo_Value,
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct MsgKVMapOO {
  pub entries: DynArray<MsgKVOO,2>,
}
