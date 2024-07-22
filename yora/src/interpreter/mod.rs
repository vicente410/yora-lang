use std::collections::HashMap;
use std::io::stdin;
use std::io::Write;
use std::process;

use crate::core::PrimitiveType;
use crate::syntax_analysis::parser::expression::*;
use crate::syntax_analysis::parser::statement::*;

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Int(i64),
    Bool(bool),
    Char(char),
    Array(Vec<Value>),
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
    fn get_array(&self) -> Vec<Value> {
        match self {
            Value::Array(array) => array.clone(),
            _ => panic!("Not a string"),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Signal {
    Normal,
    Continue,
    Break,
    Return(Value),
}

pub struct Interpreter {
    variables: Vec<(String, Value)>,
    num_vars_scope: Vec<i8>,
    procedures: HashMap<String, StatementKind>,
    signal: Signal,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            variables: Vec::new(),
            num_vars_scope: Vec::new(),
            procedures: HashMap::new(),
            signal: Signal::Normal,
        }
    }

    pub fn run(&mut self, ast: &Vec<Statement>) {
        self.start_scope();
        for statement in ast {
            self.run_statement(statement);
        }
        self.end_scope();
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

    fn run_call(&mut self, name: &String, call_args: &[Expression]) {
        self.start_scope();
        match name.as_str() {
            "exit" => {
                process::exit(0);
            }
            "print" => {
                match self.eval_expression(&call_args[0]) {
                    Value::Int(int) => print!("{}", int),
                    Value::Bool(boolean) => print!("{}", boolean),
                    Value::Char(character) => print!("{}", character),
                    Value::Array(values) => {
                        for value in values {
                            print!("{}", value.get_char());
                        }
                    }
                };
                let _ = std::io::stdout().flush();
            }
            "input" => {
                let mut buffer = String::new();
                let mut values = Vec::new();

                let _ = stdin().read_line(&mut buffer);
                let _ = buffer.strip_suffix('\n');
                for ch in buffer.chars() {
                    values.push(Value::Char(ch))
                }
                self.signal = Signal::Return(Value::Array(values));
            }
            "string_to_int" => {
                if let Value::Array(values) = self.eval_expression(&call_args[0]) {
                    let mut string = String::new();
                    for value in values {
                        string.push(value.get_char());
                    }
                    string.pop();
                    self.signal = Signal::Return(Value::Int(string.parse().unwrap()))
                } else {
                    panic!()
                }
            }
            "char_to_int" => {
                if let Value::Char(ch) = self.eval_expression(&call_args[0]) {
                    self.signal = Signal::Return(Value::Int(ch as i64))
                } else {
                    panic!()
                }
            }
            "int_to_char" => {
                if let Value::Int(int) = self.eval_expression(&call_args[0]) {
                    self.signal = Signal::Return(Value::Char(char::from_u32(int as u32).unwrap()))
                } else {
                    panic!()
                }
            }
            "string_len" => {
                if let Value::Array(contents) = self.eval_expression(&call_args[0]) {
                    self.signal = Signal::Return(Value::Int(contents.len() as i64))
                } else {
                    panic!()
                }
            }
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
                    let len = self.num_vars_scope.len();
                    self.num_vars_scope[len - 1] += 1;
                    self.variables.push((arg_name.to_string(), arg_val));
                }
                for statement in block {
                    self.run_statement(statement);
                    if let Signal::Return(_) = &self.signal {
                        break;
                    }
                }
            }
        }
        self.end_scope();
    }

    fn run_declare(&mut self, name: &String, value: &Option<Expression>) {
        let len = self.num_vars_scope.len();
        self.num_vars_scope[len - 1] += 1;
        if let Some(value) = value {
            let value = self.eval_expression(value);
            self.variables.push((name.to_string(), value));
        } else {
            self.variables.push((name.to_string(), Value::Int(0)));
        }
    }

    fn run_assign(&mut self, dest: &Expression, src: &Expression) {
        match &dest.kind {
            ExpressionKind::Id(id) => {
                let src_val = self.eval_expression(src);
                self.set_value_by_name(id, src_val);
            }
            ExpressionKind::Call(name, args) => {
                if name == "[]" {
                    let idx = self.eval_expression(&args[1]).get_int();
                    if let ExpressionKind::Id(id) = &args[0].kind {
                        let mut new_array = self.get_value_by_name(id).get_array();
                        new_array[idx as usize] = self.eval_expression(src);
                        self.set_value_by_name(id, Value::Array(new_array))
                    }
                } else {
                    panic!("Invalid assignment");
                }
            }
            _ => panic!("Invalid assignment"),
        }
    }

    fn run_if(&mut self, cond: &Expression, block: &Vec<Statement>) {
        self.start_scope();
        if let Value::Bool(cond) = self.eval_expression(cond) {
            if cond {
                for statement in block {
                    self.run_statement(statement);
                }
            }
        }
        self.end_scope();
    }

    fn run_if_else(
        &mut self,
        cond: &Expression,
        true_block: &Vec<Statement>,
        false_block: &Vec<Statement>,
    ) {
        self.start_scope();
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
        self.end_scope();
    }

    fn run_loop(&mut self, block: &Vec<Statement>) {
        self.start_scope();
        'outer: loop {
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
        }
        self.end_scope();
    }

    fn run_while(&mut self, cond: &Expression, block: &Vec<Statement>) {
        self.start_scope();
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
        self.end_scope();
    }

    fn eval_expression(&mut self, expr: &Expression) -> Value {
        match &expr.kind {
            ExpressionKind::Id(id) => self.get_value_by_name(id),
            ExpressionKind::Lit(lit) => match &expr.r#type.clone().unwrap() {
                PrimitiveType::Int => Value::Int(lit.parse::<i64>().unwrap()),
                PrimitiveType::Bool => Value::Bool(lit == "true"),
                PrimitiveType::Char => Value::Char(lit.chars().nth(1).unwrap()),
                PrimitiveType::Arr(r#type) => match **r#type {
                    PrimitiveType::Char => {
                        let mut chars = Vec::new();
                        let mut char_it = lit[1..lit.len() - 1].chars();
                        while let Some(ch) = char_it.next() {
                            if ch == '\\' {
                                chars.push(Value::Char(match char_it.next().unwrap() {
                                    'n' => '\n',
                                    't' => '\t',
                                    '0' => '\0',
                                    '\\' => '\\',
                                    _ => panic!("Unterminated escape code"),
                                }))
                            } else {
                                chars.push(Value::Char(ch));
                            }
                        }
                        Value::Array(chars)
                    }
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
                        "[]" => {
                            if let ExpressionKind::Id(id) = &args[0].kind {
                                if let Value::Array(values) = &self.get_value_by_name(id) {
                                    values[arg1.get_int() as usize].clone()
                                } else {
                                    panic!("Not an array");
                                }
                            } else {
                                panic!("Not an array")
                            }
                        }
                        _ => self.run_call_expr(name, args),
                    }
                } else if args.len() == 1 {
                    match name.as_str() {
                        "!" => Value::Bool(!self.eval_expression(&args[0]).get_bool()),
                        "-" => Value::Int(-self.eval_expression(&args[0]).get_int()),
                        _ => self.run_call_expr(name, args),
                    }
                } else {
                    self.run_call_expr(name, args)
                }
            }
            ExpressionKind::Array(contents) => {
                let mut values = Vec::new();

                for expr in contents {
                    values.push(self.eval_expression(expr));
                }

                Value::Array(values)
            }
        }
    }

    fn run_call_expr(&mut self, name: &String, args: &[Expression]) -> Value {
        self.run_call(name, args);
        let ret_value = if let Signal::Return(value) = &self.signal {
            value.clone()
        } else {
            panic!("Procedure '{name}' did not return")
        };
        self.signal = Signal::Normal;
        ret_value
    }

    fn get_value_by_name(&self, name: &String) -> Value {
        let vars = self.variables.iter().rev();
        for var in vars {
            if var.0 == *name {
                return var.1.clone();
            }
        }
        panic!("Undeclared variable '{name}' used");
    }

    fn set_value_by_name(&mut self, name: &String, value: Value) {
        let vars = self.variables.iter().enumerate().rev();
        for (i, var) in vars {
            if var.0 == *name {
                self.variables[i].1 = value;
                return;
            }
        }
        panic!("Undeclared variable '{name}' used");
    }

    fn start_scope(&mut self) {
        self.num_vars_scope.push(0);
    }

    fn end_scope(&mut self) {
        for _ in 0..self.num_vars_scope.pop().unwrap() {
            self.variables.pop();
        }
    }
}
