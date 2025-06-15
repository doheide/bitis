mod messages;
mod helper;

use std::collections::HashMap;
use std::env;
use std::process::ExitCode;
use ascii::AsciiString;
use bitis_lib::{BitisAString, DynArray, IntegerBaseFunctions, VarWithGivenBitSize};
use messages::*;

fn main() -> ExitCode {
    let mut error_counter = 0;

    println!("key-value message test!");

    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);

    // *****************************************************
    // ***
    let msg = MsgKVMapSimple::default();
    let fn_name = "val_kv_simple_default.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let mut msg = MsgKVMapSimple::default();
    msg.entries.val.push(MsgKVSimple{
        key: BitisAString::from_str(AsciiString::from_ascii("lala").unwrap()),
        value: BitisAString::from_str(AsciiString::from_ascii("val1").unwrap()),
    });
    msg.entries.val.push(MsgKVSimple{
        key: BitisAString::from_str(AsciiString::from_ascii("lili").unwrap()),
        value: BitisAString::from_str(AsciiString::from_ascii("valval2").unwrap()),
    });
    let fn_name = "val_kv_simple_val1.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    let kv_map: HashMap<String, String> = msg.entries.val.iter().map(|v| {
        (v.key.get_string(), v.value.get_string())
    }).collect();
    println!("kv_map_simple: {:?}", kv_map);

    // *****************************************************
    // ***
    let msg = MsgKVMapOO::default();
    let fn_name = "val_kv_oo_default.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    // ***
    let mut msg = MsgKVMapOO::default();
    msg.entries.val.push(MsgKVOO{
        key: AsciiString::from_ascii("lala").unwrap().into(),
        value: OO_MsgKvoo_Value::IntVal(312.into()),
    });
    msg.entries.val.push(MsgKVOO{
        key: AsciiString::from_ascii("lili").unwrap().into(),
        value: OO_MsgKvoo_Value::NumVal(0.56789.into()),
    });
    msg.entries.val.push(MsgKVOO{
        key: AsciiString::from_ascii("lolo").unwrap().into(),
        value: OO_MsgKvoo_Value::StrVal(AsciiString::from_ascii("val1").unwrap().into()),
    });
    let fn_name = "val_kv_oo_val1.rs.dat";
    error_counter += helper::write_or_test(fn_name, &msg, &args);

    let kv_map: HashMap<String, OO_MsgKvoo_Value> = msg.entries.val.iter().map(|v| {
        (v.key.get_string(), v.value.clone())
    }).collect();
    println!("kv_map_oo: {:?}", kv_map);


    println!("\n* Total errors: {}", error_counter);
    ExitCode::from(error_counter)
}
