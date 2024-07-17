use core::panic;
use std::collections::HashMap;

use crate::core::PrimitiveType;
use crate::expression::*;
use crate::statement::*;

#[derive(Clone)]
enum Value {
    Int(i64),
    Bool(bool),
    Char(char),
    String(String),
}

impl Value {
    fn get_int(&self) -> i64 {
        match self {
            Value::Int(int) => *int,
            _ => panic!("Not an int"),
        }
    }
    fn get_bool(&self) -> bool {
        match self {
            Value::Bool(bool) => *bool,
            _ => panic!("Not an bool"),
        }
    }
}

pub struct Interpreter {
    values: HashMap<String, Value>,
    //procedures: HashMap<String, Statement>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            values: HashMap::new(),
        }
    }

    pub fn run(&mut self, ast: &Vec<Statement>) {
        for statement in ast {
            self.run_statement(statement);
        }
    }

    fn run_statement(&mut self, statement: &Statement) {
        match &statement.kind {
            StatementKind::Procedure {
                name,
                args,
                ret,
                block,
            } => todo!(),
            StatementKind::Call { name, args } => todo!(),
            StatementKind::Return { value } => todo!(),
            StatementKind::Declare { name, value, .. } => {
                if let Some(value) = value {
                    let value = self.eval_expression(&value);
                    self.values.insert(name.to_string(), value);
                } else {
                    self.values.insert(name.to_string(), Value::Int(0));
                }
            }
            StatementKind::Assign { dest, src } => todo!(),
            StatementKind::If { cond, block } => self.run_if(cond, block),
            StatementKind::IfElse {
                cond,
                true_block,
                false_block,
            } => self.run_if_else(cond, true_block, false_block),
            StatementKind::Loop { block } => todo!(),
            StatementKind::While { cond, block } => todo!(),
            StatementKind::Continue => todo!(),
            StatementKind::Break => todo!(),
        }
    }

    fn run_if(&mut self, cond: &Expression, block: &Vec<Statement>) {
        if let Value::Bool(cond) = self.eval_expression(cond) {
            if cond {
                for statement in block {
                    self.run_statement(statement);
                }
            }
        }
    }

    fn run_if_else(
        &mut self,
        cond: &Expression,
        true_block: &Vec<Statement>,
        false_block: &Vec<Statement>,
    ) {
        if let Value::Bool(cond) = self.eval_expression(cond) {
            if cond {
                for statement in true_block {
                    self.run_statement(statement);
                }
            } else {
                for statement in false_block {
                    self.run_statement(statement);
                }
            }
        }
    }

    fn run_if(&mut self, cond: &Expression, block: &Vec<Statement>) {
        if let Value::Bool(cond) = self.eval_expression(cond) {
            if cond {
                for statement in block {
                    self.run_statement(statement);
                }
            }
        }
    }

    fn run_if(&mut self, cond: &Expression, block: &Vec<Statement>) {
        if let Value::Bool(cond) = self.eval_expression(cond) {
            if cond {
                for statement in block {
                    self.run_statement(statement);
                }
            }
        }
    }

    fn eval_expression(&mut self, expr: &Expression) -> Value {
        match &expr.kind {
            ExpressionKind::Id(id) => self.values[id].clone(),
            ExpressionKind::Lit(lit) => match &expr.r#type.clone().unwrap() {
                PrimitiveType::Int => Value::Int(lit.parse::<i64>().unwrap()),
                PrimitiveType::Bool => Value::Bool(if lit == "true" { true } else { false }),
                PrimitiveType::Char => Value::Char(lit.chars().nth(1).unwrap()),
                PrimitiveType::Arr(r#type) => match **r#type {
                    PrimitiveType::Char => Value::String(lit.to_string()),
                    _ => panic!("Literal arrays can only be of type Char[]"),
                },
            },
            ExpressionKind::Call(name, args) => {
                if args.len() == 2 {
                    let arg0 = self.eval_expression(&args[0]);
                    let arg1 = self.eval_expression(&args[1]);

                    match name.as_str() {
                        "+" => Value::Int(arg0.get_int() + arg1.get_int()),
                        "-" => Value::Int(arg0.get_int() - arg1.get_int()),
                        "*" => Value::Int(arg0.get_int() * arg1.get_int()),
                        "/" => Value::Int(arg0.get_int() / arg1.get_int()),
                        "%" => Value::Int(arg0.get_int() % arg1.get_int()),
                        "and" => Value::Bool(arg0.get_bool() && arg1.get_bool()),
                        "or" => Value::Bool(arg0.get_bool() || arg1.get_bool()),
                        "==" => Value::Bool(arg0.get_int() == arg1.get_int()),
                        "!=" => Value::Bool(arg0.get_int() != arg1.get_int()),
                        "<" => Value::Bool(arg0.get_int() < arg1.get_int()),
                        "<=" => Value::Bool(arg0.get_int() <= arg1.get_int()),
                        ">" => Value::Bool(arg0.get_int() > arg1.get_int()),
                        ">=" => Value::Bool(arg0.get_int() >= arg1.get_int()),
                        _ => todo!(),
                    }
                } else if args.len() == 1 {
                    let arg0 = self.eval_expression(&args[0]);
                    match name.as_str() {
                        "!" => Value::Bool(!arg0.get_bool()),
                        _ => todo!(),
                    }
                } else {
                    todo!();
                }
            }
            ExpressionKind::Array(..) => todo!(),
        }
    }
}
