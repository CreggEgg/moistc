use std::fs::{self, File};

use clap::Parser;
use lalrpop_util::lalrpop_mod;

mod compiler;
mod parser;

#[cfg(test)]
mod test;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of times to greet
    mode: String,

    /// File to work on
    file: String,
}

lalrpop_mod!(grammar);

fn main() {
    let args = Args::parse();
    let file = fs::read_to_string(args.file).expect("Failed to read from file");
    match &args.mode[..] {
        "lex" => {
            dbg!(parser::extract_funcs(file));
        }
        "exec" => {
            let mut compiler = compiler::Compiler::new();
            compiler.compile_program(parser::extract_funcs(file));
            compiler.exec();
        }
        unknown => println!("Unknown compiler command: {}", unknown),
    }
}
