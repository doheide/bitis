mod simple_msg;

use simple_msg;

fn main() {
    println!("- Initializin msg");

    let msg = MsgSimpleTest{};
    println!("{:?}", msg);
}

