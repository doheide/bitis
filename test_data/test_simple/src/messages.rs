use bitis_lib::BiserdiMsg;

// Enums

// Enums for oneof

// Messages

#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct InnerMsg {
  fix_attr: bool,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct Lala {
  data_bool: [bool;10],
  data1_uint: u8,
  data2_uint: u16,
  inner: InnerMsg,
}
