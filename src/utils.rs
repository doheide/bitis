use console::style;
use std::process::exit;


/// Prints `error: msg` to stderr and exits with code 1.
pub fn print_error(msg: String) -> ! {
    eprintln!("{} {}", style("error:").bold().red(), msg);
    exit(1);
}
pub fn print_warn(msg: String) {
    eprintln!("{} {}", style("warning:").bold().yellow(), msg);
}
