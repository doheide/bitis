use std::fmt::Debug;
use std::fs;
use bitis_lib::{deserialize, serialize, BiserdiTrait};

pub fn write_or_test<T: BiserdiTrait+Debug+PartialEq>(fn_name: &str, msg: &T, args: &Vec<String>) {
    if args.len() <= 1 {
        println!("* writing {}", fn_name);
        let data = serialize(msg);
        fs::write(fn_name, data).expect(format!("Unable to write file: {}", fn_name).as_str());
    }
    else {
        let fn_name = fn_name.replace(".rs.", format!(".{}.", args[1]).as_str());
        println!("* reading {}", fn_name);
        let data = fs::read(fn_name.clone()).expect(format!("Unable to write file: {}", fn_name).as_str());
        let msg_read = match deserialize::<T>(&data) {
            None => { panic!("!Error deserializing msg"); },
            Some((r, _s)) => { r }
        };
        println!("org: {:?}", *msg);
        println!("read: {:?}", msg_read);
        if msg_read == *msg { println!("** ok"); }
        else { println!("** failed"); }
    }
}

