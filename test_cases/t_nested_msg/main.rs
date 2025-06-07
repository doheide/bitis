mod messages;
mod helper;

use std::env;
use std::process::ExitCode;
use bitis_lib::BitisOption;
use messages::*;

fn main() -> ExitCode {
    let mut error_counter = 0;

    println!("Simple message test!");

    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);

    // ***
    let msg = MsgFixedBaseArray::default();
    let fn_name = "val_fixed_default.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgFixedBaseArray{
        param_1: SensorSource::TemperaturSensor.into(),
        val: [1.into(), 2.into(), 3.into(), 1.into(), 2.into(), 
            3.into(), 1.into(), 2.into(), 3.into(), 1.into() 
        ].into(),
    };
    let fn_name = "val_nested_val1.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);


    // ***
    println!("* Total errors: {}\n", error_counter);
    ExitCode::from(error_counter)
}
