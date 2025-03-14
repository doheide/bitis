use bitis_lib::*;
use bitis_lib::{BiserdiTrait};
use std::fmt::Debug;

mod messages;

use messages::*;


fn serialize_and_deserialize<T: BiserdiTrait + Debug>(msg: T) {
    println!("\n***");
    // serialize
    let mut ser = Biseri::new();
    msg.bit_serialize(&mut ser);
    let r = ser.finish_add_data().unwrap();
    println!("bits: {}, bytes: {}", r.0, r.1);
    println!("data: {:?}", ser.get_data());

    // deserialize
    let mut der = Bides::from_biseri(&ser.clone());
    let data = T::bit_deserialize(0, &mut der);
    println!("data: {:?}", data);
}


fn main() {
    let msg = ExampleMessage{
        is_active: false,
        value_one: 3.into(),
        signed_value: Some(2.into()),
        array: [1.into(), 2.into(), 3.into()],
    };
    serialize_and_deserialize(msg);

    let msg = BSResponseAfterHi{
        content: OO_BsResponseAfterHi_Content::challenge(1234.into())
    };
    serialize_and_deserialize(msg);

    let msg = BSResponseAfterHi{
        content: OO_BsResponseAfterHi_Content::error_reason(ErrorFlag::DeviceId_Unknown)
    };
    serialize_and_deserialize(msg);

    let msg = BSSayHiWithId{
        reserved_must_be_two: 2.into(),
        device_id: 11001.into(),
        version: 1.into(),
    };
    serialize_and_deserialize(msg);
}
