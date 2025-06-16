mod messages;
mod helper;

use std::env;
use std::process::ExitCode;
use messages::*;

fn main() -> ExitCode {
    let mut error_counter = 0;

    println!("dyn-integer message test!");

    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);

    // *****************************************************
    // ***
    let msg = MsgWithDynInt::default();
    let fn_name = "dyn_int_default.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgWithDynInt{ val: 1.into(), signed_val: 1.into() };
    let fn_name = "dyn_int_val1.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgWithDynInt{ val: 5.into(), signed_val: (-5).into() };
    let fn_name = "dyn_int_val2.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgWithDynInt{ val: 8.into(), signed_val: (-8).into() };
    let fn_name = "dyn_int_val3.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgWithDynInt{ val: 15.into(), signed_val: (-15).into() };
    let fn_name = "dyn_int_val4.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);


    println!("\n* Total errors: {}", error_counter);
    ExitCode::from(error_counter)
}
