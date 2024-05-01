// use logos::Logos;

// #[derive(Default, Debug, Clone, PartialEq)]
// enum LexingError {
//     #[default]
//     ,
// }
// pub mod ast;

use crate::grammar;

// #[derive(Logos, Debug, PartialEq)]
// #[logos()]
// pub enum Token {
//     #[regex(r#"[0-9]+"#, |lexer| lexer.slice().parse::<u64>().unwrap())]
//     Number(u64),
//
//     #[token("+")]
//     Plus,
//
//     // #[regex(r#""([^"\\]|u[a-fA-F0-9]{4})*""#, |lexer| {let slice = lexer.slice();let len = slice.len() - 1;String::from(&slice[1..len])})]
//     // Str(String),
//
//     #[regex(r"[ \t\r\n\f]+")]
//     WHITESPACE,
//
//     // #[token("save")]
//     // Save,
//     //
//     // #[token("to")]
//     // To,
//
//     #[token(".")]
//     Period,
//
//     #[token("wayto")]
//     Period,
//
//     #[regex(r#"[a-zA-Z]+"#, |lexer| lexer.slice().to_string())]
//     Ident(String),
// }
//
#[derive(Debug, Clone)]
pub enum Value {
    Number(i32),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Value(Value),
    Ident(String),
    Operation(Box<Expr>, Op, Box<Expr>),
    Def {
        ident: String,
        value: Box<Expr>,
        body: Box<Expr>,
    },
    FunctionCall(String, Vec<Box<Expr>>)
}

#[derive(Debug, Clone)]
pub struct Func {
    pub name: String,
    pub args: Vec<String>,
    pub body: Expr,
}

#[derive(Debug, Clone)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

pub fn extract_funcs(string: String) -> Vec<Func> {
    grammar::FunctionsParser::new().parse(&string).unwrap()
    // let lexer = Token::lexer(&string);
    // lexer
    //     .enumerate()
    //     .map(|(i, x)| {
    //         let x = x.unwrap();
    //         // println!("{i}, {:?}", &x);
    //         x
    //     })
    //     .collect::<Vec<Token>>()
}
