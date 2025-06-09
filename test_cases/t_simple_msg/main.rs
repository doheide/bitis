mod messages;
mod helper;

use std::env;
use std::process::ExitCode;
use bitis_lib::BitisOption;
use messages::*;
use crate::messages::{MsgSimpleOpt, MsgSimpleTestFp};


fn main() -> ExitCode {
    let mut error_counter = 0;

    println!("Simple message test!");

    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);

    // ***
    let msg = MsgSimpleBaseOneInt {
        param_1: 1122.into(),
    };
    let fn_name = "val_simple_one_int.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgSimpleBaseThreeInt {
        param_1: 1122.into(),
        param_2: 3.into(),
        param_3: 3.into(),
    };
    let fn_name = "val_simple_three_int.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgSimpleTestBase {
        param_1: Default::default(),
        param_2: false,
        param_3: Default::default(),
    };
    let fn_name = "val_simple_default.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgSimpleTestBase {
        param_1: 999.into(),
        param_2: true.into(),
        param_3: (-13).into(),
    };
    let fn_name = "val_simple_param_set1.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgSimpleTestFp {
        param_1: true.into(),
        fp: 0.1.into(),
    };
    let fn_name = "val_simple_test_fp.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgSimpleOpt {
        param_1: Default::default(),
        param_2: Default::default(),
        param_3: Default::default(),
        param_4: Default::default(),
    };
    let fn_name = "val_simple_opt_default.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgSimpleOpt {
        param_1: 223.into(),
        param_2: true.into(),
        param_3: Some(1234).into(),
        param_4: BitisOption::new_none()
    };
    let fn_name = "val_simple_opt_valset1.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);


    println!("\n* Total errors: {}", error_counter);
    ExitCode::from(error_counter)
}
