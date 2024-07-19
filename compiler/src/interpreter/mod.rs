use core::panic;
use std::collections::HashMap;

use crate::core::PrimitiveType;
use crate::expression::*;
use crate::statement::*;

#[derive(Clone, Debug)]
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
            _ => panic!("Not a bool"),
        }
    }
    fn get_char(&self) -> char {
        match self {
            Value::Char(char) => *char,
            _ => panic!("Not a char"),
        }
    }
    fn get_string(&self) -> String {
        match self {
            Value::String(string) => string.clone(),
            _ => panic!("Not a string"),
        }
    }
}

pub struct Interpreter {
    values: HashMap<String, Value>,
    procedures:
        HashMap<(String, Vec<PrimitiveType>), (Vec<Statement>, Option<PrimitiveType>, Vec<String>)>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            values: HashMap::new(),
            procedures: HashMap::new(),
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
            } => {
                let mut args_types: Vec<PrimitiveType> = Vec::with_capacity(args.len());
                let mut args_names: Vec<String> = Vec::with_capacity(args.len());
                for arg in args {
                    if let Some(arg_type) = &arg.1 {
                        args_types.push(arg_type.clone());
                    } else {
                        panic!("Insert type hint in procedure declaration");
                    }
                    args_names.push(arg.0.clone())
                }

                self.procedures.insert(
                    (name.to_string(), args_types),
                    (block.to_vec(), ret.clone(), args_names),
                );
            }
            StatementKind::Call { name, args } => {
                self.run_call(name.to_string(), args.to_vec());
            }
            StatementKind::Return { value } => {
                let ret_value = self.eval_expression(value);
                self.values.insert("@acc".to_string(), ret_value);
            }
            StatementKind::Declare { name, value, .. } => {
                if let Some(value) = value {
                    let value = self.eval_expression(&value);
                    self.values.insert(name.to_string(), value);
                } else {
                    self.values.insert(name.to_string(), Value::Int(0));
                }
            }
            StatementKind::Assign { dest, src } => {
                if let ExpressionKind::Id(id) = &dest.kind {
                    let src_value = self.eval_expression(src);
                    self.values.insert(id.to_string(), src_value);
                }
            }
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

    fn run_call(&mut self, name: String, args: Vec<Expression>) {
        let mut args_types: Vec<PrimitiveType> = Vec::with_capacity(args.len());
        let mut changed_vars: Vec<(String, Value)> = Vec::with_capacity(args.len());

        for arg in &args {
            let arg_val = self.eval_expression(&arg);
            args_types.push(self.get_val_type(arg_val));
        }

        for (i, name) in self.procedures[&(name.clone(), args_types.clone())]
            .2
            .clone()
            .into_iter()
            .enumerate()
        {
            if self.values.contains_key(&name) {
                changed_vars.push((name.clone(), self.values[&name].clone()));
            }

            let arg_value = self.eval_expression(&args[i]);
            self.values.insert(name.clone(), arg_value);
        }

        for statement in self.procedures[&(name.clone(), args_types.clone())]
            .0
            .clone()
        {
            self.run_statement(&statement);
        }

        for var in changed_vars {
            self.values.insert(var.0.clone(), var.1);
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
                        _ => {
                            self.run_call(name.to_string(), args.to_vec());
                            self.values["@acc"].clone()
                        }
                    }
                } else if args.len() == 1 {
                    let arg0 = self.eval_expression(&args[0]);
                    match name.as_str() {
                        "!" => Value::Bool(!arg0.get_bool()),
                        _ => {
                            self.run_call(name.to_string(), args.to_vec());
                            self.values["@acc"].clone()
                        }
                    }
                } else {
                    self.run_call(name.to_string(), args.to_vec());
                    self.values["@acc"].clone()
                }
            }
            ExpressionKind::Array(..) => todo!(),
        }
    }

    fn get_val_type(&mut self, val: Value) -> PrimitiveType {
        match val {
            Value::Int(_) => PrimitiveType::Int,
            Value::Bool(_) => PrimitiveType::Bool,
            Value::Char(_) => PrimitiveType::Char,
            Value::String(_) => PrimitiveType::Arr(Box::new(PrimitiveType::Char)),
        }
    }
}
