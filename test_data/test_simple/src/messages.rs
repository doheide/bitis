use bitis_lib::*;

// Enums

#[derive(BiserdiEnum, Debug, Clone, PartialEq)]
#[biserdi_enum_id_dynbits(3)]
#[allow(nonstandard_style)]
pub enum ErrorFlag {
  DeviceId_Unknown,
  Version_Unknown,
}

// Enums for oneof

#[derive(BiserdiOneOf, Debug, Clone, PartialEq)]
#[biserdi_enum_id_dynbits(2)]
#[allow(nonstandard_style)]
pub enum OO_BsResponseAfterHi_Content {
  challenge(VarWithGivenBitSize<u16, 16>),
  error_reason(ErrorFlag),
}#[derive(BiserdiOneOf, Debug, Clone, PartialEq)]
#[biserdi_enum_id_dynbits(2)]
#[allow(nonstandard_style)]
pub enum OO_BsFinalSession_Content {
  session_id(VarWithGivenBitSize<u32, 32>),
  error(Error_WrongChallengeResponse),
}

// Messages

#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct ExampleMessage {
  pub is_active: bool,
  pub value_one: VarWithGivenBitSize<u8, 3>,
  pub signed_value: Option<VarWithGivenBitSize<i8, 5>>,
  pub array: [VarWithGivenBitSize<u8, 3>;3],
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct BSSayHiWithId {
  pub reserved_must_be_two: VarWithGivenBitSize<u8, 3>,
  pub device_id: VarWithGivenBitSize<u32, 32>,
  pub version: VarWithGivenBitSize<u8, 5>,
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct BSResponseAfterHi {
  pub content: OO_BsResponseAfterHi_Content,
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct BSChallengeResponse {
  pub proof: [VarWithGivenBitSize<u8, 8>;4],
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct Error_WrongChallengeResponse {
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct BSFinalSession {
  pub content: OO_BsFinalSession_Content,
}
