mod simple_msg;
use bitis_lib::VarWithGivenBitSize;
use bitis_lib::Biseri;
use bitis_lib::BiserdiTrait;

fn main() {
    // ***
    println!("- Initializin msg");

    let msg = simple_msg::MsgSimpleTest{param_1: 7.into(), param_2: true, param_3: VarWithGivenBitSize{val: -11}};
    println!("{:?}", msg);

    // ***
    println!("- Serializing msg");
    let mut ser = Biseri::new();
    msg.bit_serialize(&mut ser);
    let r = ser.finish_add_data().unwrap();
    println!("bits: {}, bytes: {}", r.0, r.1);
    println!("data: {:?}", ser.get_data());
}

