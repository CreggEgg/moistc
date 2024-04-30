use logos::Logos;

// #[derive(Default, Debug, Clone, PartialEq)]
// enum LexingError {
//     #[default]
//     ,
// }
pub mod ast;

#[derive(Logos, Debug, PartialEq)]
#[logos()]
pub enum Token {
    #[regex(r#"[0-9]+"#, |lexer| lexer.slice().parse::<u64>().unwrap())]
    Number(u64),

    #[token("+")]
    Plus,

    #[regex(r#""([^"\\]|u[a-fA-F0-9]{4})*""#, |lexer| {let slice = lexer.slice();let len = slice.len() - 1;String::from(&slice[1..len])})]
    Str(String),

    #[regex(r"[ \t\r\n\f]+")]
    WHITESPACE,

    #[token("save")]
    Save,

    #[token("to")]
    To,

    #[token(".")]
    Period,

    #[regex(r#"[a-zA-Z]+"#, |lexer| lexer.slice().to_string())]
    Ident(String),
}
pub fn extract_tokens(string: String) -> Vec<Token> {
    let lexer = Token::lexer(&string);
    lexer
        .enumerate()
        .map(|(i, x)| {
            let x = x.unwrap();
            // println!("{i}, {:?}", &x);
            x
        })
        .collect::<Vec<Token>>()
}
