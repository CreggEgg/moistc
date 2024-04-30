use cranelift::codegen::gimli::write::Expression;

use super::Token;

#[derive(Debug)]
enum Value {
    Number(u64),
    String(String),
    Unit,
}

#[derive(Debug)]
enum Operator {
    Add,
}

#[derive(Debug)]
enum Expr {
    Value(Value),
    Variable(String),
    Operator {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        op: Operator,
    },
}

#[derive(Debug)]
pub enum Node {
    Expression(Expr),
    Binding { name: String, value: Expr },
}

impl From<u64> for Expr {
    fn from(value: u64) -> Self {
        Expr::Value(Value::Number(value))
    }
}
impl From<Expr> for Node {
    fn from(value: Expr) -> Self {
        Self::Expression(value)
    }
}

pub fn generate_ast(tokens: Vec<Token>) -> Vec<Node> {
    // let mut ast_builder = AstBuilder::new(tokens);
    // ast_builder.build()
    vec![parse_node(&tokens).unwrap().1]
}

type ParseResult<'a, T> = Result<(&'a [Token], T), AstError<'a>>;

pub fn parse_expression(tokens: &[Token]) -> ParseResult<Expr> {
    match tokens.get(0).ok_or(AstError::EOI)? {
        Token::Number(num) => {
            dbg!(tokens);
            let (tokens, _) = consume_whitespace(&tokens[1..])?;
            let operator = tokens.get(1).map(token_to_operator);
            let (tokens, rhs) = parse_expression(&tokens[1..])?;
            Ok((
                tokens,
                if let Some(Some(op)) = operator {
                    Expr::Operator {
                        lhs: Box::new(Expr::from(*num)),
                        rhs: Box::new(rhs),
                        op,
                    }
                } else {
                    Expr::from(*num)
                },
            ))
        }
        Token::Plus => todo!(),
        Token::Str(_) => todo!(),
        Token::WHITESPACE => todo!(),
        Token::Save => todo!(),
        Token::To => todo!(),
        Token::Period => todo!(),
        Token::Ident(_) => todo!(),
    }
}
pub fn parse_node(tokens: &[Token]) -> ParseResult<Node> {
    match tokens.get(0).ok_or(AstError::EOI)? {
        Token::Number(_) | Token::Str(_) => {
            parse_expression(tokens).map(|(remaining, x)| (remaining, Node::from(x)))
        }
        // Token::Str(_) => parse_expression(tokens).map(|(remaining, x)| (remaining, Node::from(x))),
        Token::Save => parse_save(tokens),
        x => Err(AstError::UnexpectedToken(x)),
    }
}

pub fn parse_save(tokens: &[Token]) -> ParseResult<Node> {
    match tokens.get(0).ok_or(AstError::EOI)? {
        Token::Save => {
            let (tokens, _) = consume_whitespace_required(&tokens[1..])?;
            let (tokens, expr) = parse_expression(tokens)?;
            if Token::To == *tokens.get(1).ok_or(AstError::EOI)? {
                let (tokens, _) = consume_whitespace_required(&tokens[1..])?;
                if let Token::Ident(ident) = tokens.get(2).ok_or(AstError::EOI)? {
                    Ok((
                        &tokens[1..],
                        Node::Binding {
                            name: ident.to_string(),
                            value: expr,
                        },
                    ))
                } else {
                    Err(AstError::ExpectedToken(Token::Ident("".to_string())))
                }
            } else {
                Err(AstError::ExpectedToken(Token::To))
            }
        }
        x => Err(AstError::UnexpectedToken(x)),
    }
}

fn consume_whitespace(tokens: &[Token]) -> ParseResult<Vec<&Token>> {
    let whitespace_tokens = tokens
        .iter()
        .take_while(|x| **x == Token::WHITESPACE)
        .collect::<Vec<&Token>>();
    Ok((&tokens[whitespace_tokens.len()..], whitespace_tokens))
}
fn consume_whitespace_required(tokens: &[Token]) -> ParseResult<Vec<&Token>> {
    let whitespace_tokens = tokens
        .iter()
        .take_while(|x| **x == Token::WHITESPACE)
        .collect::<Vec<&Token>>();
    if whitespace_tokens.is_empty() {
        return Err(AstError::ExpectedToken(Token::WHITESPACE));
    }
    Ok((&tokens[whitespace_tokens.len()..], whitespace_tokens))
}

#[derive(Debug)]
enum AstError<'a> {
    NoNextToken,
    EOI,
    UnexpectedToken(&'a Token),
    ExpectedToken(Token),
}

// struct AstBuilder {
//     nodes: Vec<Node>,
//     tokens: Vec<Token>,
//     position: usize,
// }
//
// impl AstBuilder {
//     pub fn new(tokens: Vec<Token>) -> Self {
//         Self {
//             nodes: Vec::new(),
//             tokens,
//             position: 0,
//         }
//     }
//
//     pub fn parse_node(&mut self) -> Result<Node, AstError> {
//         let (current_token, next_token, builder) = self.advance()?;
//
//         Ok(match current_token {
//             Token::Number(x) => {
//                 if let Some(op) = token_to_operator(*next_token?) {
//                     let (_, _, builder) = builder.advance()?;
//                     let rhs = node_to_expression(builder.parse_node()?);
//                     Node::Expression(Expr::Operator {
//                         lhs: Box::new(Expr::Value(Value::Number(*x))),
//                         rhs: Box::new(rhs),
//                         op: Operator::Add,
//                     })
//                 } else {
//                     Node::Expression(Expr::Value(Value::Number(*x)))
//                 }
//             }
//             Token::Plus => todo!(),
//             Token::Str(_) => todo!(),
//             Token::WHITESPACE => todo!(),
//             Token::Save => todo!(),
//             Token::To => todo!(),
//             Token::Period => todo!(),
//             Token::Ident => todo!(),
//         })
//     }
//
//     pub fn peek(&self) -> Result<&Token, AstError> {
//         self.tokens
//             .get(self.position + 1)
//             .ok_or(AstError::NoNextToken)
//     }
//     pub fn advance(&mut self) -> Result<(&Token, Result<&Token, AstError>, &mut Self), AstError> {
//         self.position += 1;
//
//         let current_token = self.tokens.get(self.position).ok_or(AstError::EOI)?;
//         let next_token = self.peek();
//         Ok((current_token, next_token, self))
//     }
//
//     pub fn build(self) -> Vec<Node> {
//         self.nodes
//     }
// }
//
pub fn node_to_expression(node: Node) -> Expr {
    match node {
        Node::Expression(x) => x,
        Node::Binding { name: _, value: _ } => Expr::Value(Value::Unit),
    }
}

pub fn token_to_operator(token: &Token) -> Option<Operator> {
    match token {
        Token::Plus => Some(Operator::Add),
        _ => None,
    }
}
