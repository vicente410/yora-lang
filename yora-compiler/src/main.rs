use std::env;
use std::fs;

#[derive(Debug)]
enum Token {
    Return,
    IntLit(i32),
    SemiColon,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Incorrect usage, gain some skill and come back later.");
    }

    let source = fs::read_to_string(&args[1]).unwrap();

    lexer(source);
}

fn lexer(source: String) {
    let mut tokens: Vec<Token> = Vec::new();

    if &source[0..6] == "return" {
        tokens.push(Token::Return);
    }

    let mut i = 0;
    let mut j = 0;

    while j != tokens.len() {
        tokens.push();
    }

    dbg!(tokens);
}

fn get_token(string: &str) -> Option<Token> {
    match string.parse() {
        Ok(num) => return Some(Token::IntLit(num)),
        Err(_) => {}
    };

    match string {
        "return" => Some(Token::Return),
        ";" => Some(Token::SemiColon),
        _ => None,
    }
}
