mod messages;
mod helper;

use std::env;
use std::process::ExitCode;
use bitis_lib::BitisOption;
use messages::{MsgEnumOne, MsgEnumTwo, SensorSource};
use crate::messages::{ExampleEnum, MsgEnumOpt};

fn main() -> ExitCode {
    let mut error_counter = 0;

    println!("Simple message test!");

    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);

    // ***
    let msg = MsgEnumOne{ val: Default::default(), param_1: SensorSource::TemperaturSensor };
    let fn_name = "val_enum_one_default.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgEnumOne{ val: 3.into(), param_1: SensorSource::MovementSensor };
    let fn_name = "val_enum_one_val1.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgEnumTwo{ val: Default::default(), param_1: Default::default() };
    let fn_name = "val_enum_two_default.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgEnumTwo{ val: 33.into(), param_1: ExampleEnum::E8 };
    let fn_name = "val_enum_two_val1.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgEnumOpt{ val: Default::default(), param_1: Default::default(), param_2: Default::default(), };
    let fn_name = "val_enum_opt_default.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgEnumOpt{ val: 3.into(), param_1: SensorSource::TemperaturSensor, 
        param_2: Some(ExampleEnum::E8).into(), };
    let fn_name = "val_enum_opt_val1.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    println!("\n* Total errors: {}", error_counter);
    ExitCode::from(error_counter)
}
