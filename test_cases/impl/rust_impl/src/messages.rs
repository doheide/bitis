use bitis_lib::*;

// Enums
#[derive(BiserdiEnum, Debug, Clone, PartialEq, Default)]
#[biserdi_enum_id_dynbits(3)]
#[allow(nonstandard_style)]
pub enum SensorSource {
  #[default] TemperaturSensor,
  MovementSensor,
}#[derive(BiserdiEnum, Debug, Clone, PartialEq, Default)]
#[biserdi_enum_id_dynbits(2)]
#[allow(nonstandard_style)]
pub enum ExampleEnum {
  E1,
  E2,
  #[default] E3,
  E4,
  E5,
  E6,
  E7,
  E8,
  E9,
}

// Enums for oneof


// Messages
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct MsgEnumOpt {
  pub val: VarWithGivenBitSize<u8, 3>,
  pub param_1: SensorSource,
  pub param_2: BitisOption<ExampleEnum>,
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct MsgWithInner {
  pub val: VarWithGivenBitSize<u8, 3>,
  pub imsg: MsgEnumOpt,
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct MsgWithTwoInner {
  pub val: VarWithGivenBitSize<u8, 3>,
  pub imsg: MsgWithInner,
  pub oimsg: BitisOption<MsgEnumOpt>,
}
