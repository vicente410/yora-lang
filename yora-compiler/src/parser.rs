use std::process;

use crate::lexer::Token;

#[derive(Debug, PartialEq)]
pub enum Expression {
    // Literals
    Identifier(String),
    BoolLit(String),
    IntLit(String),

    // Control flow
    Sequence(Vec<Expression>),
    If(Box<Expression>, Box<Expression>),
    Loop(Box<Expression>),
    Break,

    // Variables
    Declare(Box<Expression>, Box<Expression>),
    Assign(Box<Expression>, Box<Expression>),

    // Built-in functions
    Exit(Box<Expression>),

    // Operators
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Mod(Box<Expression>, Box<Expression>),
    Eq(Box<Expression>, Box<Expression>),
    NotEq(Box<Expression>, Box<Expression>),
    Less(Box<Expression>, Box<Expression>),
    LessEq(Box<Expression>, Box<Expression>),
    Greater(Box<Expression>, Box<Expression>),
    GreaterEq(Box<Expression>, Box<Expression>),
    /*Declaration(Identifier, Option<Expression>),
    Print(Box<Expression>),
    LoopBlock(Vec<Statement>),*/
    /*Not(&'a Expression),
    And(&'a Expression, &'a Expression),
    Or(&'a Expression, &'a Expression),
    Xor(&'a Expression, &'a Expression),*/
}

pub fn parse(tokens: Vec<Token>) -> Vec<Expression> {
    vec![get_sequence(&tokens[..])]
}

fn get_sequence(tokens: &[Token]) -> Expression {
    let mut sequence: Vec<Expression> = Vec::new();
    let mut start = 0;
    let mut end = 0;

    while end + 1 < tokens.len() {
        if matches!(tokens[start], Token::If) || matches!(tokens[start], Token::Loop) {
            while end < tokens.len() && !matches!(tokens[end], Token::OpenCurly) {
                end += 1;
            }

            let mut curly_counter = 1;

            while end + 1 < tokens.len() && curly_counter != 0 {
                end += 1;
                if matches!(tokens[end], Token::OpenCurly) {
                    curly_counter += 1;
                } else if matches!(tokens[end], Token::CloseCurly) {
                    curly_counter -= 1;
                }
            }
        } else {
            while end + 1 < tokens.len() && !matches!(tokens[end], Token::SemiColon) {
                end += 1;
            }
        }
        sequence.push(get_expression(&tokens[start..end]));
        start = end + 1;
        end += 1;
    }

    Expression::Sequence(sequence)
}

fn get_expression(tokens: &[Token]) -> Expression {
    let len = tokens.len();

    if len == 1 {
        return match &tokens[0] {
            Token::Identifier(id) => Expression::Identifier(id.to_string()),
            Token::BoolLit(bool) => Expression::BoolLit(bool.to_string()),
            Token::IntLit(int) => Expression::IntLit(int.to_string()),
            Token::Break => Expression::Break,
            _ => panic!("Unrecognized expression."),
        };
    }

    match (&tokens[1], &tokens[len - 1]) {
        (Token::OpenParen, Token::CloseParen) => match tokens[0] {
            Token::Exit => Expression::Exit(Box::new(get_expression(&tokens[2..len - 1]))),
            /*Token::Print => {
                Expression::Print(Box::new(get_expression(&tokens[2..len - 1].to_vec())))
            }*/
            _ => panic!("Unrecognized expression."),
        },
        _ => {
            if tokens[0] == Token::Var && tokens[2] == Token::Assign {
                Expression::Declare(
                    Box::new(get_expression(&tokens[1..2])),
                    Box::new(get_expression(&tokens[3..])),
                )
            } else if tokens[0] == Token::If {
                let mut i = 0;
                while i < len && tokens[i] != Token::OpenCurly {
                    i += 1;
                }
                Expression::If(
                    Box::new(get_expression(&tokens[1..i])),
                    Box::new(get_sequence(&tokens[i + 1..])),
                )
            } else if tokens[0] == Token::Loop {
                Expression::Loop(Box::new(get_sequence(&tokens[2..])))
            } else if tokens[1] == Token::Assign {
                Expression::Assign(
                    Box::new(get_expression(&tokens[0..1])),
                    Box::new(get_expression(&tokens[2..])),
                )
            } else if tokens[2] == Token::Assign {
                match tokens[1] {
                    Token::Add | Token::Sub | Token::Mul | Token::Div | Token::Mod => {
                        let mut new_tokens = Vec::from(tokens);
                        new_tokens.remove(1);
                        new_tokens.insert(2, tokens[0].clone());
                        new_tokens.insert(3, tokens[1].clone());
                        get_expression(&new_tokens)
                    }
                    _ => panic!("Unrecognized assign operation"),
                }
            } else {
                match &tokens[len - 2] {
                    Token::Add
                    | Token::Sub
                    | Token::Mul
                    | Token::Div
                    | Token::Mod
                    | Token::Eq
                    | Token::NotEq
                    | Token::Less
                    | Token::LessEq
                    | Token::Greater
                    | Token::GreaterEq => {
                        let arg1 = Box::new(get_expression(&tokens[0..len - 2]));
                        let arg2 = Box::new(get_expression(&tokens[len - 1..]));
                        get_operation(&tokens[len - 2], arg1, arg2)
                    }
                    _ => {
                        println!("Unrecognized expression:");
                        dbg!(tokens);
                        process::exit(1);
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
        Token::Eq => Expression::Eq(arg1, arg2),
        Token::NotEq => Expression::NotEq(arg1, arg2),
        Token::Less => Expression::Less(arg1, arg2),
        Token::LessEq => Expression::LessEq(arg1, arg2),
        Token::Greater => Expression::Greater(arg1, arg2),
        Token::GreaterEq => Expression::GreaterEq(arg1, arg2),
        _ => panic!("Unexpected operation."),
    }
}
