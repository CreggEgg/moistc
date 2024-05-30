use std::fs::{self, File};

use clap::Parser;
use lalrpop_util::lalrpop_mod;
use target_lexicon::Triple;

use crate::compiler::types::TypeGenerator;

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
        "type" => {
            let funcs = parser::extract_funcs(file);
            let mut type_generator = TypeGenerator::new();
            dbg!(type_generator.generate_types(funcs));
        }
        "build" => {
            let compiler = compiler::Compiler::new();

            compiler.build(TypeGenerator::new().generate_types(parser::extract_funcs(file)));
        }
        unknown => println!("Unknown compiler command: {}", unknown),
    }
}
