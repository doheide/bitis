// Enums

// Enums for oneof

// Messages

#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct FixedMsg {
  fix_attr: bool,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct Lala_Base {
  data_bool: [bool;10],
  data1_uint: u8,
  data2_uint: u16,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct Lala_V1 {
  added: u8,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct Lala_V2 {
  inner_msg: Lili,
}

#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct Lili_Base {
  lili_var: u8,
}
//Automatically generated empty msg
#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct Lili_V1 {
}
//Automatically generated empty msg
#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct Lili_V2 {
}
