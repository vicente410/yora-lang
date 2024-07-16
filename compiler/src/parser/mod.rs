use std::process;

use crate::core::PrimitiveType;
use crate::expression::*;
use crate::lexer::*;

use self::statement::*;

pub mod expression;
pub mod op;
pub mod statement;

pub fn parse(tokens: Vec<Token>) -> Vec<Statement> {
    Parser::parse(&tokens)
}

pub struct Parser;

impl Parser {
    fn parse(tokens: &[Token]) -> Vec<Statement> {
        Self::get_sequence(&tokens)
    }

    fn get_sequence(tokens: &[Token]) -> Vec<Statement> {
        let mut sequence: Vec<Statement> = Vec::new();
        let mut start = 0;
        let mut end = 1;

        if tokens.len() == 1 {
            sequence.push(Self::get_statement(tokens));
        }
        while end < tokens.len() {
            if tokens[start].str == "if"
                || tokens[start].str == "loop"
                || tokens[start].str == "while"
                || tokens[start].str == "pr"
            {
                // find last token in the block
                while end < tokens.len()
                    && (tokens[end].str == "else" || tokens[start].col < tokens[end].col)
                {
                    end += 1;
                }

                sequence.push(match tokens[start].str.as_str() {
                    "if" => Self::get_if(&tokens[start..end]),
                    "loop" => Self::get_loop(&tokens[start..end]),
                    "while" => Self::get_while(&tokens[start..end]),
                    "pr" => Self::get_procedure(&tokens[start..end]),
                    _ => panic!(),
                })
            } else {
                while end < tokens.len() && tokens[start].line == tokens[end].line {
                    end += 1;
                }
                sequence.push(Self::get_statement(&tokens[start..end]));
            }

            start = end;
            end += 1;
        }

        sequence
    }

    fn get_if(tokens: &[Token]) -> Statement {
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

        Statement::new(
            if tokens[end_seq].str == "else" {
                StatementKind::IfElse {
                    cond: Self::get_expression(&tokens[start..start_seq]),
                    true_body: Self::get_sequence(&tokens[start_seq + 1..end_seq]),
                    false_body: Self::get_else(&tokens[end_seq..]),
                }
            } else {
                StatementKind::If {
                    cond: Self::get_expression(&tokens[start..start_seq]),
                    body: Self::get_sequence(&tokens[start_seq + 1..]),
                }
            },
            &tokens[0],
        )
    }

    fn get_else(tokens: &[Token]) -> Vec<Statement> {
        match tokens[1].str.as_str() {
            "if" => vec![Self::get_if(tokens)],
            ":" => Self::get_sequence(&tokens[2..]),
            _ => {
                dbg!(tokens);
                panic!("Throw a useful error");
            }
        }
    }

    fn get_loop(tokens: &[Token]) -> Statement {
        if tokens[1].str != ":" {
            panic!("Throw a useful error");
        }
        Statement::new(
            StatementKind::Loop {
                body: Self::get_sequence(&tokens[2..]),
            },
            &tokens[0],
        )
    }

    fn get_while(tokens: &[Token]) -> Statement {
        let mut start_seq = 0;

        // find end of condition
        while start_seq < tokens.len() && tokens[start_seq].str != ":" {
            start_seq += 1;
        }

        Statement::new(
            StatementKind::While {
                cond: Self::get_expression(&tokens[1..start_seq]),
                body: Self::get_sequence(&tokens[start_seq + 1..]),
            },
            &tokens[start_seq],
        )
    }

    fn get_procedure(tokens: &[Token]) -> Statement {
        let mut start_seq = 0;
        let mut args = Vec::new();

        // find end of signature
        while start_seq < tokens.len() && tokens[start_seq].str != ")" {
            start_seq += 1;
        }

        // parse args
        for arg in tokens[3..start_seq].chunks(4) {
            args.push((arg[0].str.clone(), PrimitiveType::from_str(&arg[2].str)));
        }

        Statement::new(
            StatementKind::Procedure {
                name: tokens[1].str.to_string(),
                args,
                body: Self::get_sequence(&tokens[start_seq + 2..]),
            },
            &tokens[0],
        )
    }

    fn get_statement(tokens: &[Token]) -> Statement {
        let len = tokens.len();

        if len == 1 {
            Statement::new(
                if tokens[0].str == "continue" {
                    StatementKind::Continue
                } else if tokens[0].str == "break" {
                    StatementKind::Break
                } else {
                    println!("unrecognized expression:");
                    dbg!(tokens);
                    process::exit(1);
                },
                &tokens[0],
            )
        } else if tokens[0].str == "var" {
            let type_hint = if tokens.len() > 3 && &tokens[2].str == ":" {
                Some(PrimitiveType::from_str(&tokens[3].str))
            } else {
                None
            };

            let value = if tokens.len() > 5 && tokens[4].str == "=" {
                Some(Self::get_expression(&tokens[5..]))
            } else if tokens.len() > 3 && tokens[2].str == "=" {
                Some(Self::get_expression(&tokens[3..]))
            } else {
                None
            };

            Statement::new(
                StatementKind::Declare {
                    name: tokens[1].str.clone(),
                    type_hint,
                    value,
                },
                &tokens[0],
            )
        } else if tokens[1].str == "(" && tokens[len - 1].str == ")" {
            let mut args = Vec::new();
            let mut buffer = Vec::new();
            let mut num_paren = 0;
            for token in &tokens[2..] {
                match token.str.as_str() {
                    "," => {
                        if num_paren == 0 {
                            args.push(Self::get_expression(&buffer));
                            buffer.clear()
                        } else {
                            buffer.push(token.clone());
                        }
                    }
                    "(" => {
                        buffer.push(token.clone());
                        num_paren += 1;
                    }
                    ")" => {
                        if num_paren > 0 {
                            buffer.push(token.clone());
                            num_paren -= 1;
                        }
                    }
                    _ => buffer.push(token.clone()),
                }
            }
            args.push(Self::get_expression(&buffer));
            Statement::new(
                StatementKind::Call {
                    name: tokens[0].str.clone(),
                    args,
                },
                &tokens[0],
            )
        } else if tokens[0].str == "return" {
            Statement::new(
                StatementKind::Return {
                    value: Self::get_expression(&tokens[1..]),
                },
                &tokens[0],
            )
        } else {
            for (i, token) in tokens.iter().enumerate() {
                match token.str.as_str() {
                    "=" | "+=" | "-=" | "*=" | "/=" | "%=" => {
                        return Self::get_assign(i, tokens);
                    }
                    _ => continue,
                }
            }

            println!("unrecognized expression:");
            dbg!(tokens);
            process::exit(1);
        }
    }

    fn get_assign(assign_pos: usize, tokens: &[Token]) -> Statement {
        match tokens[assign_pos].str.as_str() {
            "=" => Statement::new(
                StatementKind::Assign {
                    dest: Self::get_expression(&tokens[0..assign_pos]),
                    src: Self::get_expression(&tokens[assign_pos + 1..]),
                },
                &tokens[0],
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
                Self::get_statement(&new_tokens)
            }
            _ => panic!("Not an assign operation"),
        }
    }

    fn get_expression(tokens: &[Token]) -> Expression {
        let len = tokens.len();

        if len == 1 {
            return Expression::new(
                match &tokens[0].kind {
                    TokenKind::Identifier
                    | TokenKind::BoolLit
                    | TokenKind::IntLit
                    | TokenKind::StringLit => ExpressionKind::Lit(tokens[0].str.to_string()),
                    _ => {
                        println!("Unrecognized expression:");
                        dbg!(tokens);
                        process::exit(1);
                    }
                },
                &tokens[0],
            );
        } else if tokens[1].str == "(" && tokens[len - 1].str == ")" {
            let mut args = Vec::new();
            let mut buffer = Vec::new();
            for token in &tokens[2..] {
                match token.str.as_str() {
                    "," | ")" => {
                        args.push(Self::get_expression(&buffer));
                        buffer.clear()
                    }
                    _ => buffer.push(token.clone()),
                }
            }
            Expression::new(
                ExpressionKind::Call(tokens[0].str.to_string(), args),
                &tokens[0],
            )
        } else {
            let mut pos = 0;
            let mut priority = 0;
            for (i, token) in tokens.iter().enumerate() {
                if token.str == "(" {
                    return Self::get_parentheses(tokens);
                }
                if token.kind == TokenKind::Operator && priority <= Self::get_op_priority(token) {
                    pos = i;
                    priority = Self::get_op_priority(token);
                }
            }
            if priority != 0 {
                let arg1 = Self::get_expression(&tokens[0..pos]);
                let arg2 = Self::get_expression(&tokens[pos + 1..]);
                Self::get_operation(&tokens[pos], arg1, arg2)
            } else {
                println!("Unrecognized expression:");
                dbg!(tokens);
                process::exit(1);
            }
        }
    }

    fn get_parentheses(tokens: &[Token]) -> Expression {
        let mut start = 0;
        while start < tokens.len() && tokens[start].str != "(" {
            start += 1;
        }
        let mut end = start + 1;
        let mut num_paren = 1;
        while end < tokens.len() && num_paren > 0 {
            if tokens[end].str == "(" {
                num_paren += 1;
            } else if tokens[end].str == ")" {
                num_paren -= 1;
            }
            end += 1;
        }
        if num_paren != 0 {
            panic!("Unmatched parentheses");
        }

        let mut expr = Self::get_expression(&tokens[start + 1..end - 1]);
        if start > 1 {
            let left_of_paren = Self::get_expression(&tokens[0..start - 1]);
            expr = Self::get_operation(&tokens[start - 1], left_of_paren, expr);
        }
        if end < tokens.len() {
            let right_of_paren = Self::get_expression(&tokens[end + 1..]);
            expr = Self::get_operation(&tokens[end], expr, right_of_paren);
        }
        return expr;
    }

    fn get_operation(operation: &Token, arg1: Expression, arg2: Expression) -> Expression {
        Expression::new(
            ExpressionKind::Call(operation.str.clone(), vec![arg1, arg2]),
            &operation,
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
}
