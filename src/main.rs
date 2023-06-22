extern crate getopts;

use getopts::Options;
use std::{env, fs::File};

mod graph;
fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.reqopt("f", "file", "makefile to run", "FILE");
    opts.optopt("j", "", "number of threads", "THREADS");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", opts.short_usage(&args[0]));
            panic!("{}", f);
        }
    };

    let file_name = match matches.opt_str("f") {
        Some(f) => f,
        None => "".to_string(), // branch will never be reached
    };

    let f = match File::open(file_name) {
        Ok(f) => f,
        Err(e) => panic!("{}", e),
    };

    let threads: u8 = match matches.opt_get::<u8>("j") {
        Ok(x) => match x {
            Some(y) => y,
            None => panic!("Could not find integer"),
        },
        Err(e) => panic!("{}", e),
    };
}
