mod messages;
mod helper;

use std::env;
use std::process::ExitCode;
use bitis_lib::BitisOption;
use messages::*;
use crate::messages::{ExampleEnum, MsgEnumOpt};

fn main() -> ExitCode {
    let mut error_counter = 0;

    println!("Simple message test!");

    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);

    // ***
    let inner = MsgEnumOpt::default();
    let msg = MsgWithInner::default();
    let fn_name = "val_nested_default.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let inner = MsgEnumOpt{
        val: 1.into(),
        param_1: SensorSource::TemperaturSensor.into(),
        param_2: Some(ExampleEnum::E3).into(),
    };
    let msg = MsgWithInner{ val: 2.into(), imsg: inner };
    let fn_name = "val_nested_val1.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);


    // ***
    println!("\n* Total errors: {}", error_counter);
    ExitCode::from(error_counter)
}
