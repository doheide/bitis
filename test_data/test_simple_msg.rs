// Enums

// Enums for oneof

// Messages
#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct Lala_V2 {
  data_bool: [bool;10],
  data1_uint: u8,
  data2_uint: u16,
  added: u8,
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct Lala_V0 {
  data_bool: [bool;10],
  data1_uint: u8,
  data2_uint: u16,
}
