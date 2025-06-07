use bitis_lib::*;

// Enums
#[derive(BiserdiEnum, Debug, Clone, PartialEq, Copy, Default)]
#[biserdi_enum_id_dynbits(3)]
#[allow(nonstandard_style)]
pub enum SensorSource {
  #[default] TemperaturSensor,
  MovementSensor,
}#[derive(BiserdiEnum, Debug, Clone, PartialEq, Copy, Default)]
#[biserdi_enum_id_dynbits(2)]
#[allow(nonstandard_style)]
pub enum ExampleEnum {
  E1,
  E2,
  #[default] E3,
  E4,
  E5,
  E6,

// Enums for oneof


// Messages
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct MsgFixedBaseArray {
  pub param_1: SensorSource,
  pub val1: FixedArray<VarWithGivenBitSize<u8, 3>,3>,
  pub val2: FixedArray<VarWithGivenBitSize<i8, 3>,3>,
  pub val3: FixedArray<bool,3>,
  pub val4: FixedArray<DynInteger<i8, 3>,3>,
  pub val5: FixedArray<f64,3>,
  pub val6: FixedArray<FixPrecisionMinMax<10, -2, 3>,3>,
  pub val7: FixedArray<SensorSource,3>,
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct MsgDynBaseArray {
  pub ee: ExampleEnum,
  pub val1: DynArray<VarWithGivenBitSize<u8, 3>,3>,
  pub val2: DynArray<VarWithGivenBitSize<i8, 3>,3>,
  pub val3: DynArray<bool,3>,
  pub val4: DynArray<DynInteger<u8, 3>,3>,
  pub val5: DynArray<f64,3>,
  pub val6: DynArray<FixPrecisionMinMax<10, -2, 3>,3>,
  pub val7: DynArray<SensorSource,6>,
}
