mod eval;
mod parser;

use std::env;

use eval::{EvalResult, eval_expr};
use parser::parse;

fn main() {
    let mut verbose = false;
    let mut show_help = false;
    let mut expr_parts = Vec::new();

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "-v" | "--verbose" => verbose = true,
            "-h" | "--help" => show_help = true,
            _ => expr_parts.push(arg.to_lowercase()),
        }
    }

    if show_help || expr_parts.is_empty() {
        eprintln!(
            "Usage: roll [options] <expr>\n\n\
             Options:\n\
             \t-v, --verbose   Show individual rolls\n\
             \t-h, --help      Show this help message"
        );
        std::process::exit(if show_help { 0 } else { 1 });
    }

    let input = expr_parts.join(" ");
    let expressions = match parse(&input) {
        Ok(exprs) => exprs,
        Err(err) => {
            eprintln!("Parse error: {}", err);
            std::process::exit(1);
        }
    };

    for expr in expressions.iter() {
        let eval = eval_expr(expr);
        if verbose {
            match eval {
                EvalResult::Rolls(v) => println!("{:?}", v),
                EvalResult::Number(n) => println!("{}", n),
            }
        } else {
            println!("{}", eval.to_number());
        }
    }
}
