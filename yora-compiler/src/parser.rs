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
    Literal(String),
    Identifier(String),
    /*Not(&'a Expression),
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
    match buffer.len() {
        2 => match (&buffer[0], &buffer[1]) {
            (Token::Exit, Token::Integer(int)) => {
                Statement::Exit(Expression::Literal(int.to_string()))
            }
            (Token::Exit, Token::Identifier(id)) => {
                Statement::Exit(Expression::Identifier(id.to_string()))
            }
            (Token::Print, Token::Integer(int)) => {
                Statement::Print(Expression::Literal(int.to_string()))
            }
            (Token::Print, Token::Identifier(id)) => {
                Statement::Print(Expression::Identifier(id.to_string()))
            }
            _ => panic!("Unrecognized statement."),
        },
        3 => match (&buffer[0], &buffer[1], &buffer[2]) {
            (Token::Identifier(id), Token::Equal, Token::Integer(int)) => Statement::Assignment(
                Expression::Identifier(id.to_string()),
                Expression::Literal(int.to_string()),
            ),
            (Token::Identifier(id1), Token::Equal, Token::Identifier(id2)) => {
                Statement::Assignment(
                    Expression::Identifier(id1.to_string()),
                    Expression::Identifier(id2.to_string()),
                )
            }
            _ => panic!("Unrecognized statement."),
        },
        _ => {
            panic!("Unrecognized statement.");
        }
    }
}
