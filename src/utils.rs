use console::{style, /*Emoji*/};
use std::process::exit;


pub fn print_error(msg: String) -> ! {
    println!("\n{} {}", style("Error:").bold().red(), msg);
    exit(-1);
}
#[allow(dead_code)]
pub fn print_warn(msg: String) {
    println!("\n{} {}", style("Warning:").bold().yellow(), msg);
}
