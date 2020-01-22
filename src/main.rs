extern crate regex;
pub mod vt100_string;

fn main() {
    let st = vt100_string::slice("Hello, world", 1, -1);
    println!("{}", st);
}
