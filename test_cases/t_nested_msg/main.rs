mod messages;
mod helper;

use std::env;
use std::process::ExitCode;
use messages::*;

fn main() -> ExitCode {
    let mut error_counter = 0;

    println!("Nested message test!");

    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);

    // ***
    let msg = MsgWithInner::default();
    let fn_name = "val_nested_default.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let inner = MsgEnumOpt{
        val: 1.into(),
        param_1: SensorSource::TemperaturSensor.into(),
        param_2: Some(ExampleEnum::E3).into(),
    };
    let msg = MsgWithInner{
        val: 2.into(),
        imsg: inner,
    };
    let fn_name = "val_nested_val1.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let inner = MsgEnumOpt{
        val: 1.into(),
        param_1: SensorSource::TemperaturSensor.into(),
        param_2: Some(ExampleEnum::E3).into(),
    };
    let msgi = MsgWithInner{
        val: 2.into(),
        imsg: inner,
    };
    let msg = MsgWithTwoInner{
        val: 47.into(),
        imsg: msgi,
        oimsg: bitis_lib::BitisOption::<MsgEnumOpt>::new_none(),
    };
    let fn_name = "val_nested_two_val1.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);


    // ***
    println!("* Total errors: {}\n", error_counter);
    ExitCode::from(error_counter)
}
