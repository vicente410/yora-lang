use crate::lexer::Token;

#[derive(Debug, PartialEq)]
pub enum Statement {
    Exit(Expression),
    /*Declaration(Identifier, Option<Expression>),
    Assignment(Identifier, Expression),
    IfBlock(Expression, Vec<Statement>),
    LoopBlock(Vec<Statement>),*/
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Literal(String),
    /*Identifier(String),
    Not(&'a Expression),
    Equal(&'a Expression, &'a Expression),
    NotEqual(&'a Expression, &'a Expression),
    And(&'a Expression, &'a Expression),
    Or(&'a Expression, &'a Expression),
    Xor(&'a Expression, &'a Expression),
    Add(&'a Expression, &'a Expression),
    Sub(&'a Expression, &'a Expression),
    Mul(&'a Expression, &'a Expression),
    Div(&'a Expression, &'a Expression),
    Rem(&'a Expression, &'a Expression),*/
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

fn get_statement(buffer: &Vec<Token>) -> Statement {
    if buffer.len() == 2
        && matches!(buffer[0], Token::Exit)
        && matches!(buffer[1], Token::Integer(..))
    {
        if let Token::Integer(string) = &buffer[1] {
            return Statement::Exit(Expression::Literal(string.to_string()));
        } else {
            panic!("Unrecognized statement.");
        }
    } else {
        panic!("Unrecognized statement.");
    }
}
