use std::{env, io};
use std::io::BufRead; 
use std::fs::File;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Too few arguments.");
        process::exit(1);
    }
    let filename = &args[1];
    let file = File::open(filename).unwrap();
    println!("{}", io::BufReader::new(file).lines().count());
}
