use lexer::lex;
use lexer::Token;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;

mod lexer;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename: String;

    if args.len() != 2 {
        panic!("Incorrect usage.");
    }

    match &args[1].strip_suffix(".yo") {
        Some(res) => filename = res.to_string(),
        None => panic!("Incorrect extension: please use .yo"),
    }

    let source = fs::read_to_string(&args[1]).unwrap();

    output(lex(source), filename);
}

fn output(tokens: Vec<Token>, filename: String) {
    let mut buffer = String::new();

    buffer.push_str("global _start\n_start:");

    dbg!(&tokens);

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

    let mut file = File::create(format!("{}.asm", filename)).unwrap();
    file.write_all(&buffer.into_bytes()).unwrap();

    Command::new("nasm")
        .args(["-f elf64", &format!("{}.asm", filename)])
        .output()
        .expect("Failed to execute \"nasm\"");

    Command::new("ld")
        .args(["-o", &filename, &format!("{}.o", filename)])
        .output()
        .expect("Failed to execute \"ld\"");

    Command::new("rm")
        .args([format!("{}.o", filename)])
        .output()
        .expect("Failed to remove object file");

    println!("Successfully compiled");
}
