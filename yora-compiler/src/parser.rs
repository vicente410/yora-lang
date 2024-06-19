use crate::lexer::Token;

#[derive(Debug, PartialEq)]
pub enum Expression {
    Value(Value),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Rem(Box<Expression>, Box<Expression>),
    Exit(Box<Expression>),
    Print(Box<Expression>),
    Assign(Box<Expression>, Box<Expression>),
    Sequence(Vec<Expression>),
    /*Declaration(Identifier, Option<Expression>),
    IfBlock(Expression, Expression),
    LoopBlock(Vec<Statement>),*/
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

fn get_expression(tokens: &Vec<Token>) -> Expression {
    let len = tokens.len();

    if len == 1 {
        return match &tokens[0] {
            Token::Identifier(id) => Expression::Value(Value::Identifier(id.to_string())),
            Token::Integer(int) => Expression::Value(Value::Integer(int.to_string())),
            _ => panic!("Unrecognized expression."),
        };
    }

    match (&tokens[1], &tokens[len - 1]) {
        (Token::LeftParen, Token::RightParen) => match tokens[0] {
            Token::Exit => Expression::Exit(Box::new(get_expression(&tokens[2..len - 1].to_vec()))),
            Token::Print => {
                Expression::Print(Box::new(get_expression(&tokens[2..len - 1].to_vec())))
            }
            _ => panic!("Unrecognized statement."),
        },
        _ => match &tokens[len - 2] {
            Token::Add => Expression::Add(
                Box::new(get_expression(&tokens[0..len - 2].to_vec())),
                Box::new(get_expression(&tokens[len - 1..].to_vec())),
            ),
            Token::Sub => Expression::Sub(
                Box::new(get_expression(&tokens[0..len - 2].to_vec())),
                Box::new(get_expression(&tokens[len - 1..].to_vec())),
            ),
            Token::Mul => Expression::Mul(
                Box::new(get_expression(&tokens[0..len - 2].to_vec())),
                Box::new(get_expression(&tokens[len - 1..].to_vec())),
            ),
            Token::Div => Expression::Div(
                Box::new(get_expression(&tokens[0..len - 2].to_vec())),
                Box::new(get_expression(&tokens[len - 1..].to_vec())),
            ),
            Token::Rem => Expression::Rem(
                Box::new(get_expression(&tokens[0..len - 2].to_vec())),
                Box::new(get_expression(&tokens[len - 1..].to_vec())),
            ),
            _ => {
                if tokens[1] == Token::Equal {
                    Expression::Assignment(
                        Box::new(get_expression(&tokens[0..1].to_vec())),
                        Box::new(get_expression(&tokens[2..].to_vec())),
                    )
                } else {
                    panic!("Unrecognized statement.")
                }
            }
        },
    }
}
