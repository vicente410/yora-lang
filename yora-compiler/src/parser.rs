use core::panic;

use crate::lexer::Token;

#[derive(Debug, PartialEq)]
pub enum Expression {
    Identifier(String),
    BoolLit(String),
    IntLit(String),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Mod(Box<Expression>, Box<Expression>),
    Exit(Box<Expression>),
    If(Box<Expression>, Box<Expression>),
    Declaration(Box<Expression>, Box<Expression>),
    Assign(Box<Expression>, Box<Expression>),
    Sequence(Vec<Box<Expression>>),
    /*Declaration(Identifier, Option<Expression>),
    Print(Box<Expression>),
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
    let mut start = 0;
    let mut end = 0;

    while end < tokens.len() {
        if matches!(tokens[end], Token::SemiColon) {
            ast.push(get_expression(&tokens[start..=end]));
            start = end + 1;
        } else if matches!(tokens[end], Token::If) {
            while end < tokens.len() && matches!(tokens[end], Token::CloseCurly) {
                end += 1;
            }
            ast.push(get_expression(&tokens[start..=end]));
            start = end + 1;
        }
        end += 1;
    }

    ast
}

fn get_expression(tokens: &[Token]) -> Expression {
    let len = tokens.len();

    if len == 1 {
        return match &tokens[0] {
            Token::Identifier(id) => Expression::Identifier(id.to_string()),
            Token::BoolLit(int) => Expression::BoolLit(int.to_string()),
            Token::IntLit(int) => Expression::IntLit(int.to_string()),
            _ => panic!("Unrecognized expression."),
        };
    }
    dbg!(tokens);

    match (&tokens[1], &tokens[len - 2]) {
        (Token::OpenParen, Token::CloseParen) => match tokens[0] {
            Token::Exit => Expression::Exit(Box::new(get_expression(&tokens[2..len - 2]))),
            /*Token::Print => {
                Expression::Print(Box::new(get_expression(&tokens[2..len - 1].to_vec())))
            }*/
            _ => panic!("Unrecognized expression."),
        },
        _ => {
            if tokens[0] == Token::Var && tokens[2] == Token::Equal {
                Expression::Declaration(
                    Box::new(get_expression(&tokens[1..2])),
                    Box::new(get_expression(&tokens[3..len - 1])),
                )
            } else if tokens[0] == Token::If {
                let mut i = 0;
                while i < len && tokens[i] != Token::OpenCurly {
                    i += 1;
                }
                Expression::If(
                    Box::new(get_expression(&tokens[1..i])),
                    Box::new(get_expression(&tokens[i + 1..len - 1])),
                )
            } else if tokens[1] == Token::Equal {
                Expression::Assign(
                    Box::new(get_expression(&tokens[0..1])),
                    Box::new(get_expression(&tokens[2..len - 1])),
                )
            } else {
                match &tokens[len - 2] {
                    Token::Add | Token::Sub | Token::Mul | Token::Div | Token::Mod => {
                        let arg1 = Box::new(get_expression(&tokens[0..len - 3]));
                        let arg2 = Box::new(get_expression(&tokens[len - 2..len - 1]));
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
            Token::OpenParen,
            Token::IntLit("2".to_string()),
            Token::Add,
            Token::IntLit("3".to_string()),
            Token::CloseParen,
            Token::SemiColon,
        ];
        let output = vec![Expression::Exit(Box::new(Expression::Add(
            Box::new(Expression::IntLit("2".to_string())),
            Box::new(Expression::IntLit("3".to_string())),
        )))];
        assert_eq!(parse(input), output);
    }
}
