use std::fmt::Debug;
use std::fs;
use bitis_lib::{deserialize, serialize, BiserdiTrait};

pub fn write_or_test<T: BiserdiTrait+Debug+PartialEq>(fn_name: &str, msg: &T, args: &Vec<String>) -> u8 {
    if args.len() <= 1 {
        println!("* writing {}", fn_name);
        let (data, r) = serialize(msg);
        println!("{:?}", r);
        fs::write(fn_name, data).expect(format!("Unable to write file: {}", fn_name).as_str());
        0
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
        println!("\nread: {:?}", msg_read);
        if msg_read == *msg { println!("** ok\n"); 0 }
        else { println!("** failed\n"); 1 }
    }
}

