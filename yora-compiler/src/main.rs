use lexer::lex;
use lexer::Token;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;

mod lexer;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Incorrect usage, gain some skill and come back later.");
    }

    let source = fs::read_to_string(&args[1]).unwrap();

    output(lex(source));
}

fn output(tokens: Vec<Token>) {
    let mut buffer = String::new();
    let mut file = File::create("a.out").unwrap();

    buffer.push_str("global _start\n_start:");

    if tokens.len() == 3
        && tokens[0] == Token::Return
        && matches!(tokens[1], Token::Integer(..))
        && tokens[2] == Token::SemiColon
    {
        buffer.push_str("    mov rax, 60\n    mov rdi, ");
        if let Token::Integer(string) = &tokens[1] {
            buffer.push_str(&string);
        }
        buffer.push_str("\n    syscall");
    } else {
        panic!("Syntax Error");
    }

    file.write_all(&buffer.into_bytes()).unwrap();
    println!("Successfully compiled");
}
