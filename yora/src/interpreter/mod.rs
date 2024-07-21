use std::collections::HashMap;
use std::process;

use crate::core::PrimitiveType;
use crate::expression::*;
use crate::statement::*;

#[derive(Clone, PartialEq)]
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

#[derive(PartialEq)]
enum Signal {
    Normal,
    Continue,
    Break,
    Return(Value),
}

pub struct Interpreter {
    values: HashMap<String, Value>,
    procedures: HashMap<String, StatementKind>,
    signal: Signal,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            values: HashMap::new(),
            procedures: HashMap::new(),
            signal: Signal::Normal,
        }
    }

    pub fn run(&mut self, ast: &Vec<Statement>) {
        for statement in ast {
            self.run_statement(statement);
        }
    }

    fn run_statement(&mut self, statement: &Statement) {
        match &statement.kind {
            StatementKind::Procedure { name, .. } => {
                self.procedures
                    .insert(name.to_string(), statement.kind.clone());
            }
            StatementKind::Call { name, args } => self.run_call(name, args),
            StatementKind::Return { value } => {
                self.signal = Signal::Return(self.eval_expression(value))
            }
            StatementKind::Declare { name, value, .. } => self.run_declare(name, value),
            StatementKind::Assign { dest, src } => self.run_assign(dest, src),
            StatementKind::If { cond, block } => self.run_if(cond, block),
            StatementKind::IfElse {
                cond,
                true_block,
                false_block,
            } => self.run_if_else(cond, true_block, false_block),
            StatementKind::Loop { block } => self.run_loop(block),
            StatementKind::While { cond, block } => self.run_while(cond, block),
            StatementKind::Continue => self.signal = Signal::Continue,
            StatementKind::Break => self.signal = Signal::Break,
        }
    }

    fn run_call(&mut self, name: &String, call_args: &Vec<Expression>) {
        match name.as_str() {
            "exit" => {
                println!(
                    "Process exited with exit code {}",
                    self.eval_expression(&call_args[0]).get_int()
                );
                process::exit(0);
            }
            "print" => match self.eval_expression(&call_args[0]) {
                Value::String(string) => println!("{}", string.trim_matches('\"')),
                Value::Int(int) => println!("{}", int),
                Value::Bool(boolean) => println!("{}", boolean),
                Value::Char(character) => println!("{}", character),
            },
            _ => {
                let StatementKind::Procedure {
                    ref args,
                    ref block,
                    ..
                } = self.procedures[name].clone()
                else {
                    panic!("Undeclared function {name}");
                };

                for (i, (arg_name, _)) in args.iter().enumerate() {
                    let arg_val = self.eval_expression(&call_args[i]);
                    self.values.insert(arg_name.to_string(), arg_val);
                }
                for statement in block {
                    self.run_statement(&statement);
                    if let Signal::Return(_) = &self.signal {
                        break;
                    }
                }
            }
        }
    }

    fn run_declare(&mut self, name: &String, value: &Option<Expression>) {
        if let Some(value) = value {
            let value = self.eval_expression(&value);
            self.values.insert(name.to_string(), value);
        } else {
            self.values.insert(name.to_string(), Value::Int(0));
        }
    }

    fn run_assign(&mut self, dest: &Expression, src: &Expression) {
        match &dest.kind {
            ExpressionKind::Id(id) => {
                let src_val = self.eval_expression(src);
                self.values.insert(id.to_string(), src_val);
            }
            ExpressionKind::Call(name, args) => {
                if name == "[]" {
                    todo!();
                } else {
                    panic!("Invalid assignment");
                }
            }
            _ => panic!("Invalid assignment"),
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
                    if self.signal == Signal::Break || self.signal == Signal::Continue {
                        break;
                    }
                }
            } else {
                for statement in false_block {
                    self.run_statement(statement);
                    if self.signal == Signal::Break || self.signal == Signal::Continue {
                        break;
                    }
                }
            }
        }
    }

    fn run_loop(&mut self, block: &Vec<Statement>) {
        loop {
            for statement in block {
                self.run_statement(statement);
                if self.signal == Signal::Break {
                    self.signal = Signal::Normal;
                    break;
                } else if self.signal == Signal::Continue {
                    self.signal = Signal::Normal;
                    continue;
                }
            }
        }
    }

    fn run_while(&mut self, cond: &Expression, block: &Vec<Statement>) {
        'outer: loop {
            if let Value::Bool(cond) = self.eval_expression(cond) {
                if cond {
                    for statement in block {
                        self.run_statement(statement);
                        if self.signal == Signal::Break {
                            self.signal = Signal::Normal;
                            break 'outer;
                        } else if self.signal == Signal::Continue {
                            self.signal = Signal::Normal;
                            continue 'outer;
                        }
                    }
                } else {
                    break;
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
                    _ => panic!(
                        "Literal arrays can only be of type Char[], {}[] given",
                        r#type
                    ),
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
                        _ => self.run_call_expr(name, args),
                    }
                } else if args.len() == 1 {
                    let arg0 = self.eval_expression(&args[0]);
                    match name.as_str() {
                        "!" => Value::Bool(!arg0.get_bool()),
                        _ => self.run_call_expr(name, args),
                    }
                } else {
                    self.run_call_expr(name, args)
                }
            }
            ExpressionKind::Array(..) => todo!(),
        }
    }

    fn run_call_expr(&mut self, name: &String, args: &Vec<Expression>) -> Value {
        self.run_call(name, args);
        let ret_value = if let Signal::Return(value) = &self.signal {
            value.clone()
        } else {
            panic!("Procedure '{name}' did not return")
        };
        self.signal = Signal::Normal;
        ret_value
    }
}
