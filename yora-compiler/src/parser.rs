use std::process;

use crate::lexer::*;

#[derive(Debug, PartialEq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub line: usize,
    pub column: usize,
}

impl Expression {
    fn new(kind: ExpressionKind, line: usize, column: usize) -> Expression {
        Expression { kind, line, column }
    }
}
#[derive(Debug, PartialEq)]
pub enum ExpressionKind {
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
    Not(Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Eq(Box<Expression>, Box<Expression>),
    Neq(Box<Expression>, Box<Expression>),
    Lt(Box<Expression>, Box<Expression>),
    Leq(Box<Expression>, Box<Expression>),
    Gt(Box<Expression>, Box<Expression>),
    Geq(Box<Expression>, Box<Expression>),
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
        if matches!(tokens[start].kind, TokenKind::If)
            || matches!(tokens[start].kind, TokenKind::Loop)
            || matches!(tokens[start].kind, TokenKind::While)
        {
            while end < tokens.len() && !matches!(tokens[end].kind, TokenKind::Colon) {
                end += 1;
            }
            end += 2;

            let mut indent_level = 1;

            while end + 1 < tokens.len() && indent_level != 0 {
                end += 1;
                match tokens[end].kind {
                    TokenKind::Indent => indent_level += 1,
                    TokenKind::Dedent => indent_level -= 1,
                    _ => {}
                }
            }
        } else {
            while end + 1 < tokens.len() && !matches!(tokens[end].kind, TokenKind::NewLine) {
                end += 1;
            }
        }
        if tokens.len() != 1 && !tokens.is_empty() {
            sequence.push(Expression::new(
                get_expression(&tokens[start..end]).kind,
                tokens[start].line,
                tokens[start].column,
            ));
        }
        start = end + 1;
        end += 1;
    }

    Expression::new(ExpressionKind::Sequence(sequence), 0, 0)
}

fn get_expression(tokens: &[Token]) -> Expression {
    let len = tokens.len();

    if len == 1 {
        return Expression::new(
            match &tokens[0].kind {
                TokenKind::Identifier(id) => ExpressionKind::Identifier(id.to_string()),
                TokenKind::BoolLit(bool) => ExpressionKind::BoolLit(bool.to_string()),
                TokenKind::IntLit(int) => ExpressionKind::IntLit(int.to_string()),
                TokenKind::Break => ExpressionKind::Break,
                _ => {
                    println!("Unrecognized expression:");
                    dbg!(tokens);
                    process::exit(1);
                }
            },
            tokens[0].line,
            tokens[0].column,
        );
    }

    Expression::new(
        match &tokens[0].kind {
            TokenKind::Exit => ExpressionKind::Exit(Box::new(get_expression(&tokens[2..len - 1]))),
            TokenKind::Var => {
                if matches!(&tokens[2].kind, TokenKind::Assign) {
                    ExpressionKind::Declare(
                        Box::new(get_expression(&tokens[1..2])),
                        Box::new(get_expression(&tokens[3..])),
                    )
                } else {
                    println!("Unrecognized expression:");
                    dbg!(tokens);
                    process::exit(1);
                }
            }
            TokenKind::If => {
                let mut i = 0;
                while i < len && tokens[i].kind != TokenKind::Colon {
                    i += 1;
                }
                ExpressionKind::If(
                    Box::new(get_expression(&tokens[1..i])),
                    Box::new(get_sequence(&tokens[i + 3..])),
                )
            }
            TokenKind::Loop => ExpressionKind::Loop(Box::new(get_sequence(&tokens[4..]))),
            TokenKind::While => {
                let mut i = 0;
                while i < len && tokens[i].kind != TokenKind::Colon {
                    i += 1;
                }
                ExpressionKind::Loop(Box::new(Expression::new(
                    ExpressionKind::Sequence(vec![
                        Expression::new(
                            ExpressionKind::Not(Box::new(Expression::new(
                                ExpressionKind::If(
                                    Box::new(get_expression(&tokens[1..i])),
                                    Box::new(Expression::new(ExpressionKind::Break, 0, 0)),
                                ),
                                0,
                                0,
                            ))),
                            0,
                            0,
                        ),
                        get_sequence(&tokens[i + 3..]),
                    ]),
                    0,
                    0,
                )))
            }
            TokenKind::Not => ExpressionKind::Not(Box::new(get_expression(&tokens[1..]))),
            _ => {
                if matches!(&tokens[1].kind, TokenKind::Assign) {
                    ExpressionKind::Assign(
                        Box::new(get_expression(&tokens[0..1])),
                        Box::new(get_expression(&tokens[2..])),
                    )
                } else if matches!(&tokens[2].kind, TokenKind::Assign) {
                    match tokens[1].kind {
                        TokenKind::Add
                        | TokenKind::Sub
                        | TokenKind::Mul
                        | TokenKind::Div
                        | TokenKind::Mod => {
                            let mut new_tokens = Vec::from(tokens);
                            new_tokens.remove(1);
                            new_tokens.insert(2, tokens[0].clone());
                            new_tokens.insert(3, tokens[1].clone());
                            get_expression(&new_tokens).kind
                        }
                        _ => panic!("Unrecognized assign operation"),
                    }
                } else {
                    match &tokens[len - 2].kind {
                        TokenKind::Add
                        | TokenKind::Sub
                        | TokenKind::Mul
                        | TokenKind::Div
                        | TokenKind::Mod
                        | TokenKind::And
                        | TokenKind::Or
                        | TokenKind::Eq
                        | TokenKind::NotEq
                        | TokenKind::Less
                        | TokenKind::LessEq
                        | TokenKind::Greater
                        | TokenKind::GreaterEq => {
                            let arg1 = Box::new(get_expression(&tokens[0..len - 2]));
                            let arg2 = Box::new(get_expression(&tokens[len - 1..]));
                            get_operation(&tokens[len - 2], arg1, arg2).kind
                        }
                        _ => {
                            println!("Unrecognized expression:");
                            dbg!(tokens);
                            process::exit(1);
                        }
                    }
                }
            }
        },
        tokens[0].line,
        tokens[0].column,
    )
}

fn get_operation(operation: &Token, arg1: Box<Expression>, arg2: Box<Expression>) -> Expression {
    Expression::new(
        match operation.kind {
            TokenKind::Add => ExpressionKind::Add(arg1, arg2),
            TokenKind::Sub => ExpressionKind::Sub(arg1, arg2),
            TokenKind::Mul => ExpressionKind::Mul(arg1, arg2),
            TokenKind::Div => ExpressionKind::Div(arg1, arg2),
            TokenKind::Mod => ExpressionKind::Mod(arg1, arg2),
            TokenKind::And => ExpressionKind::And(arg1, arg2),
            TokenKind::Or => ExpressionKind::Or(arg1, arg2),
            TokenKind::Eq => ExpressionKind::Eq(arg1, arg2),
            TokenKind::NotEq => ExpressionKind::Neq(arg1, arg2),
            TokenKind::Less => ExpressionKind::Lt(arg1, arg2),
            TokenKind::LessEq => ExpressionKind::Leq(arg1, arg2),
            TokenKind::Greater => ExpressionKind::Gt(arg1, arg2),
            TokenKind::GreaterEq => ExpressionKind::Geq(arg1, arg2),
            _ => {
                println!("Unrecognized operation:");
                dbg!(operation);
                process::exit(1);
            }
        },
        operation.line,
        operation.column,
    )
}
