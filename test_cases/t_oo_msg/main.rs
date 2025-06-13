mod messages;
mod helper;

use std::env;
use std::process::ExitCode;
use bitis_lib::{DynArray, IntegerBaseFunctions, VarWithGivenBitSize};
use messages::*;

fn main() -> ExitCode {
    let mut error_counter = 0;

    println!("Simple message test!");

    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);

    // *****************************************************
    // ***
    let msg = MsgOOSimpleBase::default();
    let fn_name = "val_oosimple_default.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgOOSimpleBase {
        id: 53.into(),
        value: OO_MsgOoSimpleBase_Value::Number(1.23.into()),
    };
    let fn_name = "val_oosimple_val1.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgOOSimpleBase {
        id: 54.into(),
        value: OO_MsgOoSimpleBase_Value::Int(3.into()),
    };
    let fn_name = "val_oosimple_val2.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgOOSimpleBase {
        id: 55.into(),
        value: OO_MsgOoSimpleBase_Value::TrueFalse(true.into()),
    };
    let fn_name = "val_oosimple_val3.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // *****************************************************
    // ***
    let msg = MsgOONestedBase::default();
    let fn_name = "val_oonested_default.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgOONestedBase{
        id: 2.into(),
        value: OO_MsgOoNestedBase_Value::Inner(MsgSimpleBaseOneInt{param_1: 1111.into()}),
    };
    let fn_name = "val_oonested_val1.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // *****************************************************
    // ***
    let msg = MsgOONestedArray::default();
    let fn_name = "val_oonestedarray_default.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let mut msg = MsgOONestedArray::default();
    msg.values.val.push(MsgOONestedBase{
        id: 2.into(),
        value: OO_MsgOoNestedBase_Value::Inner(MsgSimpleBaseOneInt{param_1: 1111.into()}),
    });
    msg.values.val.push(MsgOONestedBase::default());
    msg.values.val.push(MsgOONestedBase::default());

    let fn_name = "val_oonestedarray_val1.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // *****************************************************
    // ***
    let msg = MsgRepeatedFixedOOBase::default();
    let fn_name = "val_repeatedoo_default.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgRepeatedFixedOOBase{
        id: 1.into(),
        value: [OO_MsgRepeatedFixedOoBase_Value::default(),
            OO_MsgRepeatedFixedOoBase_Value::default(),
            OO_MsgRepeatedFixedOoBase_Value::TrueFalse(true.into())
        ].into(),
    };

    let fn_name = "val_repeatedoo_val1.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // *****************************************************
    // ***
    let msg = MsgRepeatedDynOOBase::default();
    let fn_name = "val_repeateddynoo_default.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgRepeatedDynOOBase{
        id: 65.into(),
        value: [OO_MsgRepeatedDynOoBase_Value::TrueFalse(true),
            OO_MsgRepeatedDynOoBase_Value::Int(34.into()), ].into(),
    };
    let fn_name = "val_repeateddynoo_val1.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // *****************************************************
    // ***
    let msg = MsgOptionalOOBase::default();
    let fn_name = "val_optionaloo_default.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let msg = MsgOptionalOOBase{
        id: 83.into(),
        value: Some(OO_MsgOptionalOoBase_Value::TrueFalse(true)).into(),
    };
    let fn_name = "val_optionaloo_val1.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    
    println!("\n* Total errors: {}", error_counter);
    ExitCode::from(error_counter)
}
