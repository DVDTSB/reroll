mod parser;
use std::env;

use parser::parse;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("Usage: roll <expr>");
        std::process::exit(1);
    }

    let input = args.join(" ");

    match parse(&input) {
        Ok(x) => println!("{:#?}", x),
        Err(x) => eprintln!("Parse error: {}", x),
    }
}
