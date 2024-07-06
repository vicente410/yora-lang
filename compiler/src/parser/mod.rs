use std::process;

use crate::expression::*;
use crate::lexer::*;
use crate::op::*;

pub mod expression;
pub mod op;

pub fn parse(tokens: Vec<Token>) -> Vec<Expression> {
    vec![get_sequence(&tokens[..])]
}

fn get_sequence(tokens: &[Token]) -> Expression {
    let mut sequence: Vec<Expression> = Vec::new();
    let mut start = 0;
    let mut end = 1;

    if tokens.len() == 1 {
        sequence.push(Expression::new(
            get_expression(tokens).kind,
            tokens[start].line,
            tokens[start].col,
        ));
    }
    while end + 1 < tokens.len() {
        if tokens[start].str == "if" || tokens[start].str == "loop" || tokens[start].str == "while"
        {
            // find last token in the block
            while end < tokens.len()
                && (tokens[end].str == "else" || tokens[start].col < tokens[end].col)
            {
                end += 1;
            }

            sequence.push(match tokens[start].str.as_str() {
                "if" => get_if_expr(&tokens[start..end]),
                "loop" => get_loop_expr(&tokens[start..end]),
                "while" => get_while_expr(&tokens[start..end]),
                _ => panic!(),
            })
        } else {
            while end < tokens.len() && tokens[start].line == tokens[end].line {
                end += 1;
            }
            sequence.push(Expression::new(
                get_expression(&tokens[start..end]).kind,
                tokens[start].line,
                tokens[start].col,
            ));
        }

        start = end;
        end += 1;
    }

    Expression::new(ExpressionKind::Sequence(sequence), 0, 0)
}

fn get_if_expr(tokens: &[Token]) -> Expression {
    let start = if tokens[0].str == "else" { 2 } else { 1 };
    let mut start_seq = start;
    let mut end_seq = start + 1;

    // find end of condition
    while start_seq + 1 < tokens.len() && tokens[start_seq].str != ":" {
        start_seq += 1;
    }

    // find end of first block
    while end_seq + 1 < tokens.len() && tokens[0].col < tokens[end_seq].col {
        end_seq += 1;
    }

    Expression::new(
        if tokens[end_seq].str == "else" {
            ExpressionKind::IfElse(
                Box::new(get_expression(&tokens[start..start_seq])),
                Box::new(get_sequence(&tokens[start_seq + 1..end_seq])),
                Box::new(get_else_expr(&tokens[end_seq..])),
            )
        } else {
            ExpressionKind::If(
                Box::new(get_expression(&tokens[start..start_seq])),
                Box::new(get_sequence(&tokens[start_seq + 1..])),
            )
        },
        tokens[0].line,
        tokens[0].col,
    )
}

fn get_else_expr(tokens: &[Token]) -> Expression {
    if tokens[1].str != ":" {
        panic!("Throw a useful error");
    }
    if tokens[2].str == "if" {
        get_if_expr(tokens)
    } else {
        get_sequence(&tokens[2..])
    }
}

fn get_loop_expr(tokens: &[Token]) -> Expression {
    if tokens[1].str != ":" {
        panic!("Throw a useful error");
    }
    Expression::new(
        ExpressionKind::Loop(Box::new(get_sequence(&tokens[2..]))),
        tokens[0].line,
        tokens[0].col,
    )
}

fn get_while_expr(tokens: &[Token]) -> Expression {
    let mut start_seq = 0;

    // find end of condition
    while start_seq < tokens.len() && tokens[start_seq].str != ":" {
        start_seq += 1;
    }

    Expression::new(
        ExpressionKind::While(
            Box::new(get_expression(&tokens[1..start_seq])),
            Box::new(get_sequence(&tokens[start_seq + 1..])),
        ),
        tokens[start_seq].line,
        tokens[start_seq].col,
    )
}

fn get_expression(tokens: &[Token]) -> Expression {
    let len = tokens.len();

    if len == 1 {
        return Expression::new(
            match &tokens[0].kind {
                TokenKind::Identifier => ExpressionKind::Identifier(tokens[0].str.to_string()),
                TokenKind::BoolLit => ExpressionKind::BoolLit(tokens[0].str.to_string()),
                TokenKind::IntLit => ExpressionKind::IntLit(tokens[0].str.to_string()),
                TokenKind::StringLit => ExpressionKind::StringLit(tokens[0].str.to_string()),
                TokenKind::Keyword => match tokens[0].str.as_str() {
                    "continue" => ExpressionKind::Continue,
                    "break" => ExpressionKind::Break,
                    _ => {
                        println!("Unrecognized expression:");
                        dbg!(tokens);
                        process::exit(1);
                    }
                },
                _ => {
                    println!("Unrecognized expression:");
                    dbg!(tokens);
                    process::exit(1);
                }
            },
            tokens[0].line,
            tokens[0].col,
        );
    }

    if tokens[0].str.as_str() != "var" {
        for (i, token) in tokens.iter().enumerate() {
            match token.str.as_str() {
                "=" | "+=" | "-=" | "*=" | "/=" | "%=" => {
                    return get_assign(i, tokens);
                }
                _ => continue,
            }
        }
    }

    if tokens[1].str == "[" && tokens[len - 1].str == "]" {
        return Expression::new(
            ExpressionKind::Idx(
                Box::new(get_expression(&tokens[0..1])),
                Box::new(get_expression(&tokens[2..len - 1])),
            ),
            tokens[0].line,
            tokens[0].col,
        );
    }

    Expression::new(
        match tokens[0].str.as_str() {
            "exit" => ExpressionKind::Call(
                "exit".to_string(),
                Box::new(get_expression(&tokens[2..len - 1])),
            ),
            "print" => ExpressionKind::Call(
                "print".to_string(),
                Box::new(get_expression(&tokens[2..len - 1])),
            ),
            "var" => {
                if &tokens[2].str == "=" {
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
            "[" => {
                let mut contents = Vec::new();
                let mut buffer = Vec::new();
                for token in &tokens[1..] {
                    match token.str.as_str() {
                        "," | "]" => {
                            contents.push(get_expression(&buffer));
                            buffer.clear()
                        }
                        _ => buffer.push(token.clone()),
                    }
                }
                ExpressionKind::ArrayLit(contents)
            }
            "!" => ExpressionKind::Not(Box::new(get_expression(&tokens[1..]))),
            _ => {
                let mut pos = 0;
                let mut priority = 0;
                for (i, token) in tokens.iter().enumerate() {
                    if token.kind == TokenKind::Operator && priority <= get_op_priority(token) {
                        pos = i;
                        priority = get_op_priority(token);
                    }
                }
                if priority != 0 {
                    let arg1 = Box::new(get_expression(&tokens[0..pos]));
                    let arg2 = Box::new(get_expression(&tokens[pos + 1..]));
                    get_operation(&tokens[pos], arg1, arg2).kind
                } else {
                    println!("Unrecognized expression:");
                    dbg!(tokens);
                    process::exit(1);
                }
            }
        },
        tokens[0].line,
        tokens[0].col,
    )
}

fn get_assign(assign_pos: usize, tokens: &[Token]) -> Expression {
    Expression::new(
        match tokens[assign_pos].str.as_str() {
            // todo: assign might be in different position if assinging for array
            "=" => ExpressionKind::Assign(
                Box::new(get_expression(&tokens[0..assign_pos])),
                Box::new(get_expression(&tokens[assign_pos + 1..])),
            ),
            "+=" | "-=" | "*=" | "/=" | "%=" => {
                let mut new_tokens = Vec::from(tokens);
                new_tokens.remove(assign_pos);
                new_tokens.insert(
                    assign_pos,
                    Token::new(&Buffer {
                        str: String::from("="),
                        first_ch: '\0',
                        x: tokens[assign_pos].col,
                        y: tokens[assign_pos].line,
                    }),
                );
                new_tokens.insert(assign_pos + 1, tokens[0].clone());
                new_tokens.insert(
                    assign_pos + 2,
                    Token::new(&Buffer {
                        str: tokens[1].str[0..1].to_string(),
                        first_ch: '\0',
                        x: tokens[1].col,
                        y: tokens[1].line,
                    }),
                );
                get_expression(&new_tokens).kind
            }
            _ => panic!("Not an assign operation"),
        },
        tokens[0].line,
        tokens[0].col,
    )
}

fn get_operation(operation: &Token, arg1: Box<Expression>, arg2: Box<Expression>) -> Expression {
    Expression::new(
        ExpressionKind::Op(arg1, Op::from_str(&operation.str), arg2),
        operation.line,
        operation.col,
    )
}

fn get_op_priority(operation: &Token) -> u8 {
    match operation.str.as_str() {
        "*" | "/" => 1,
        "+" | "%" | "-" => 2,
        "!" => 3,
        "==" | "!=" | "<" | "<=" | ">" | ">=" => 4,
        "and" | "or" => 5,
        _ => {
            println!("Unrecognized operation:");
            dbg!(operation);
            process::exit(1);
        }
    }
}
