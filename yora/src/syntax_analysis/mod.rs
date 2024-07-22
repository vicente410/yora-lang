use analyzer::analyze;
use errors::Errors;
use lexer::lex;
use parser::parse;
use parser::statement::Statement;

pub mod analyzer;
pub mod errors;
pub mod lexer;
pub mod parser;

pub fn produce_ast(source: String) -> Vec<Statement> {
    let mut errors = Errors::new();

    let tokens = lex(source);
    let mut ast = parse(tokens);
    analyze(&mut ast, &mut errors);

    ast
}
