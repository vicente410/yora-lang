use core::panic;

use crate::lexer::Token;

#[derive(Debug, PartialEq)]
pub enum Expression {
    Identifier(String),
    Integer(String),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Mod(Box<Expression>, Box<Expression>),
    Exit(Box<Expression>),
    Declaration(Box<Expression>, Box<Expression>),
    Assign(Box<Expression>, Box<Expression>),
    /*Declaration(Identifier, Option<Expression>),
    Print(Box<Expression>),
    Sequence(Vec<Expression>),
    IfBlock(Expression, Expression),
    LoopBlock(Vec<Statement>),*/
    /*Not(&'a Expression),
    Equal(&'a Expression, &'a Expression),
    NotEqual(&'a Expression, &'a Expression),
    And(&'a Expression, &'a Expression),
    Or(&'a Expression, &'a Expression),
    Xor(&'a Expression, &'a Expression),*/
}

pub fn parse(tokens: Vec<Token>) -> Vec<Expression> {
    let mut ast: Vec<Expression> = Vec::new();
    let mut buffer: Vec<Token> = Vec::new();

    for token in tokens {
        if !matches!(token, Token::SemiColon) {
            buffer.push(token);
        } else {
            ast.push(get_expression(&buffer));
            buffer.clear();
        }
    }

    if !buffer.is_empty() {
        panic!("Missing semicolon.");
    }

    ast
}

fn get_expression(tokens: &[Token]) -> Expression {
    let len = tokens.len();

    if len == 1 {
        return match &tokens[0] {
            Token::Identifier(id) => Expression::Identifier(id.to_string()),
            Token::Integer(int) => Expression::Integer(int.to_string()),
            _ => panic!("Unrecognized expression."),
        };
    }

    match (&tokens[1], &tokens[len - 1]) {
        (Token::LeftParen, Token::RightParen) => match tokens[0] {
            Token::Exit => Expression::Exit(Box::new(get_expression(&tokens[2..len - 1]))),
            /*Token::Print => {
                Expression::Print(Box::new(get_expression(&tokens[2..len - 1].to_vec())))
            }*/
            _ => panic!("Unrecognized expression."),
        },
        _ => {
            if tokens[0] == Token::Var && tokens[2] == Token::Equal {
                Expression::Declaration(
                    Box::new(get_expression(&tokens[1..2])),
                    Box::new(get_expression(&tokens[3..])),
                )
            } else if tokens[1] == Token::Equal {
                Expression::Assign(
                    Box::new(get_expression(&tokens[0..1])),
                    Box::new(get_expression(&tokens[2..])),
                )
            } else {
                match &tokens[len - 2] {
                    Token::Add | Token::Sub | Token::Mul | Token::Div | Token::Mod => {
                        let arg1 = Box::new(get_expression(&tokens[0..len - 2]));
                        let arg2 = Box::new(get_expression(&tokens[len - 1..]));
                        get_operation(&tokens[len - 2], arg1, arg2)
                    }
                    _ => {
                        panic!("Unrecognized expression.");
                    }
                }
            }
        }
    }
}

fn get_operation(operation: &Token, arg1: Box<Expression>, arg2: Box<Expression>) -> Expression {
    match operation {
        Token::Add => Expression::Add(arg1, arg2),
        Token::Sub => Expression::Sub(arg1, arg2),
        Token::Mul => Expression::Mul(arg1, arg2),
        Token::Div => Expression::Div(arg1, arg2),
        Token::Mod => Expression::Mod(arg1, arg2),
        _ => panic!("Unexpected operation."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let input = vec![
            Token::Exit,
            Token::LeftParen,
            Token::Integer("2".to_string()),
            Token::Add,
            Token::Integer("3".to_string()),
            Token::RightParen,
            Token::SemiColon,
        ];
        let output = vec![Expression::Exit(Box::new(Expression::Add(
            Box::new(Expression::Integer("2".to_string())),
            Box::new(Expression::Integer("3".to_string())),
        )))];
        assert_eq!(parse(input), output);
    }
}
