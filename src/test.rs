use crate::parser::{extract_tokens, Token};

#[test]
fn test_expr() {
    assert_eq!(
        extract_tokens(String::from("5+5.")),
        vec![
            Token::Number(5),
            Token::Plus,
            Token::Number(5),
            Token::Period
        ]
    );
}
#[test]
fn test_string() {
    assert_eq!(
        extract_tokens(String::from(r#""hello world"."#)),
        vec![Token::Str("hello world".to_string()), Token::Period]
    );
}
