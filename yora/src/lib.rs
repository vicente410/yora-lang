use interpreter::Interpreter;
use std::fs;
use std::process;

pub mod core;
pub mod interpreter;
pub mod syntax_analysis;

pub fn run(filename: String, debug_ast: bool) {
    let source = fs::read_to_string(filename).unwrap();
    let ast = syntax_analysis::produce_ast(source);
    let mut interpreter = Interpreter::new();

    if debug_ast {
        for statement in &ast {
            print!("{}", statement);
        }
        process::exit(0);
    }

    interpreter.run(&ast);
}
