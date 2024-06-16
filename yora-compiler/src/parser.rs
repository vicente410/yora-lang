use crate::lexer::Token;

#[derive(Debug, PartialEq)]
pub enum Statement {
    Exit(Expression),
    Print(Expression),
    Assignment(Expression, Expression),
    /*Declaration(Identifier, Option<Expression>),
    IfBlock(Expression, Vec<Statement>),
    LoopBlock(Vec<Statement>),*/
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Value(Value),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Rem(Box<Expression>, Box<Expression>),
    /*Not(&'a Expression),
    Equal(&'a Expression, &'a Expression),
    NotEqual(&'a Expression, &'a Expression),
    And(&'a Expression, &'a Expression),
    Or(&'a Expression, &'a Expression),
    Xor(&'a Expression, &'a Expression),*/
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Identifier(String),
    Integer(String),
}

pub fn parse(tokens: Vec<Token>) -> Vec<Statement> {
    let mut ast: Vec<Statement> = Vec::new();
    let mut buffer: Vec<Token> = Vec::new();

    for token in tokens {
        if !matches!(token, Token::SemiColon) {
            buffer.push(token);
        } else {
            ast.push(get_statement(&buffer));
            buffer.clear();
        }
    }

    if !buffer.is_empty() {
        panic!("Missing semicolon.");
    }

    ast
}

fn get_statement(buffer: &[Token]) -> Statement {
    if buffer.len() > 1 {
        match (&buffer[1], &buffer[buffer.len() - 1]) {
            (Token::LeftParen, Token::RightParen) => match buffer[0] {
                Token::Exit => {
                    Statement::Exit(get_expression(buffer[2..buffer.len() - 1].to_vec()))
                }
                Token::Print => {
                    Statement::Print(get_expression(buffer[2..buffer.len() - 1].to_vec()))
                }
                _ => panic!("Unrecognized statement."),
            },
            _ => {
                if buffer[1] == Token::Equal {
                    Statement::Assignment(
                        get_expression(buffer[0..1].to_vec()),
                        get_expression(buffer[2..].to_vec()),
                    )
                } else {
                    panic!("Unrecognized statement.")
                }
            }
        }
    } else {
        panic!("Unrecognized statement.")
    }
}

fn get_expression(tokens: Vec<Token>) -> Expression {
    let len = tokens.len();
    match len {
        1 => match &tokens[0] {
            Token::Identifier(id) => Expression::Value(Value::Identifier(id.to_string())),
            Token::Integer(id) => Expression::Value(Value::Integer(id.to_string())),
            _ => panic!("Unrecognized expression."),
        },
        2 => panic!("Unrecognized expression."),
        _ => match &tokens[len - 2] {
            Token::Add => Expression::Add(
                Box::new(get_expression(tokens[0..len - 2].to_vec())),
                Box::new(get_expression(tokens[len - 1..].to_vec())),
            ),
            Token::Sub => Expression::Sub(
                Box::new(get_expression(tokens[0..len - 2].to_vec())),
                Box::new(get_expression(tokens[len - 1..].to_vec())),
            ),
            Token::Mul => Expression::Mul(
                Box::new(get_expression(tokens[0..len - 2].to_vec())),
                Box::new(get_expression(tokens[len - 1..].to_vec())),
            ),
            Token::Div => Expression::Div(
                Box::new(get_expression(tokens[0..len - 2].to_vec())),
                Box::new(get_expression(tokens[len - 1..].to_vec())),
            ),
            Token::Rem => Expression::Rem(
                Box::new(get_expression(tokens[0..len - 2].to_vec())),
                Box::new(get_expression(tokens[len - 1..].to_vec())),
            ),
            _ => panic!("Unrecognized expression."),
        },
    }
}
