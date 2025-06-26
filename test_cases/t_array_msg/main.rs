mod messages;
mod helper;

use std::env;
use std::process::ExitCode;
use messages::*;

fn main() -> ExitCode {
    let mut error_counter = 0;

    println!("array message test!");

    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);

    // ***
    let inner = Inner{ val2: 3.into() };
    let inner2 = Inner{ val2: 1.into() };

    // ***
    let msg = MsgFixedBaseArray::default();
    let fn_name = "val_array_default.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgFixedBaseArray{
        param_1: SensorSource::TemperaturSensor.into(),
        val1: [1_u16.into(), 2.into(), 3.into()].into(),
        val2: [(-2).into(), 2.into(), 0.into()].into(),
        val3: [true.into(), false.into(), true.into()].into(),
        val4: [(-1).into(), 123.into(), 10.into()].into(),
        val5: [1.1.into(), 2.2.into(), 123.456.into()].into(),
        val6: [1.1.into(), (-1.1).into(), 1.2.into()].into(),
        val7: [SensorSource::MovementSensor.into(), SensorSource::MovementSensor.into(), SensorSource::TemperaturSensor.into() ].into(),
        val8: [inner.clone(), inner2.clone(), inner.clone()].into(),
    };
    let fn_name = "val_array_val1.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgDynBaseArray::default();
    let fn_name = "val_dynarray_default.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let mut msg = MsgDynBaseArray{
        ee: ExampleEnum::E5.into(),
        val1: [1.into(), 2.into(), 3.into()].into(),
        val2: [(-2).into(), 2.into(), 0.into()].into(),
        val3: [true.into(), false.into(), true.into()].into(),
        val4: [(-1).into(), 123.into(), 10.into()].into(),
        val5: [1.1.into(), 2.2.into(), 123.456.into()].into(),
        val6: [1.1.into(), (-1.1).into(), 1.2.into()].into(),
        val7: [SensorSource::MovementSensor.into(), SensorSource::MovementSensor.into(), SensorSource::TemperaturSensor.into() ].into(),
        val8: [inner.clone(), inner, inner2].into(),
    };
    msg.val1.val.push(4.into());
    let fn_name = "val_dynarray_val1.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    let mut msg = MsgDynBaseArray::default();
    msg.val1 = (0..13).into_iter().map(|i| { (i&7).into() }).collect::<Vec<_>>().into();
    msg.val2 = (0..23).into_iter().map(|i| { (i&7).into() }).collect::<Vec<_>>().into();
    let fn_name = "val_dynarray_val2.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgLargeFixedArray::default();
    let fn_name = "val_large_array_default.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let mut msg = MsgLargeFixedArray::default();
    msg.val1.val.iter_mut().enumerate().for_each(|(i, v)| { *v = ((i+1) as u16 & 7).into() });
    msg.val2.val.iter_mut().enumerate().for_each(|(i, v)| {
         let ii = (i as i16) & 7; *v = ((1-(ii&1)*2)*(ii>>1)).into() });
    msg.val3.val.iter_mut().enumerate().for_each(|(i, v)| { *v = (((i as u8) & 2) == 2).into() });
    let fn_name = "val_large_array_val1.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);


    // ***
    println!("* Total errors: {}", error_counter);
    ExitCode::from(error_counter)
}
