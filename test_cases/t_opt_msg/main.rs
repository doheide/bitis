mod messages;
mod helper;

use std::env;
use std::process::ExitCode;
use bitis_lib::BitisOption;
use messages::*;

fn main() -> ExitCode {
    let mut error_counter = 0;

    println!("array message test!");

    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);

    // ***
    let msg = MsgOpt::default();
    let fn_name = "val_opt_default.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgOpt{
        param_1: SensorSource::TemperaturSensor.into(),
        val1: Some(3).into(),
        val2: Some(-2).into(),
        val3: Some(true).into(),
        val4: Some(-1).into(),
        val5: Some(1.1).into(),
        val6: Some(1.1).into(),
        val7: Some(SensorSource::MovementSensor).into(),
    };
    let fn_name = "val_opt_val1.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);




    // ***
    println!("* Total errors: {}", error_counter);
    ExitCode::from(error_counter)
}
