use std::fs::{self, File};

use clap::Parser;

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

fn main() {
    let args = Args::parse();
    let file = fs::read_to_string(args.file).expect("Failed to read from file");
    match &args.mode[..] {
        "lex" => {
            dbg!(parser::extract_tokens(file));
        }
        "ast" => {
            let tokens = parser::extract_tokens(file);
            dbg!(parser::ast::generate_ast(tokens));
        }
        unknown => println!("Unknown mode, {}", unknown),
    }
}
