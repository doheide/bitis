use bitis_lib::*;

// Enums
#[derive(BiserdiEnum, Debug, Clone, PartialEq, Copy, Default)]
#[biserdi_enum_id_dynbits(3)]
#[allow(nonstandard_style)]
pub enum SensorSource {
  #[default] TemperaturSensor,
  MovementSensor,
}
impl std::fmt::Display for SensorSource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self) }
}
#[derive(BiserdiEnum, Debug, Clone, PartialEq, Copy, Default)]
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
impl std::fmt::Display for ExampleEnum {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self) }
}


// Enums for oneof


// Messages
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct Inner {
  pub val2: IntWithGivenBitSize<i16, 3>,
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct MsgFixedBaseArray {
  pub param_1: SensorSource,
  pub val1: FixedArray<IntWithGivenBitSize<u16, 3>,3>,
  pub val2: FixedArray<IntWithGivenBitSize<i16, 3>,3>,
  pub val3: FixedArray<bool,3>,
  pub val4: FixedArray<DynInteger<i16, 8, 3>,3>,
  pub val5: FixedArray<f64,3>,
  pub val6: FixedArray<FixPrecisionMinMax<10, -2, 3>,3>,
  pub val7: FixedArray<SensorSource,3>,
  pub val8: FixedArray<Inner,3>,
}
