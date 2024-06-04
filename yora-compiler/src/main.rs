use lexer::*;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;

mod lexer;

#[derive(Eq, Debug, Hash, PartialEq)]
enum Flag {
    Output,
    Assembly,
    Tokens,
}

fn main() {
    let mut args: Vec<String> = env::args().collect();

    let (mut filename, flags) = parse_args(&mut args);

    let source = fs::read_to_string(&filename).unwrap();

    match filename.strip_suffix(".yr") {
        Some(res) => filename = res.to_string(),
        None => panic!("Incorrect extension: please use .yr"),
    }

    output(lex(source), filename, flags);
}

fn output(tokens: Vec<Token>, filename: String, flags: HashMap<Flag, Option<String>>) {
    let mut buffer = String::new();
    let mut output = filename.clone();

    buffer.push_str("global _start\n_start:");

    if flags.contains_key(&Flag::Tokens) {
        dbg!(&tokens);
        return;
    }

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

    if flags.contains_key(&Flag::Assembly) {
        return;
    }

    if flags.contains_key(&Flag::Output) {
        if let Some(string) = &flags[&Flag::Output] {
            output = string.to_string();
        }
    }

    Command::new("ld")
        .args(["-o", &output, &format!("{}.o", filename)])
        .output()
        .expect("Failed to execute \"ld\"");

    Command::new("rm")
        .args([format!("{}.o", filename), format!("{}.asm", filename)])
        .output()
        .expect("Failed to remove object file");

    println!("Successfully compiled");
}

fn parse_args(args: &mut Vec<String>) -> (String, HashMap<Flag, Option<String>>) {
    let mut flags: HashMap<Flag, Option<String>> = HashMap::new();
    let mut i = 1;
    let filename = args.pop().unwrap();

    while i < args.len() {
        flags.insert(
            match args[i].as_str() {
                "-o" | "--output" => Flag::Output,
                "-s" | "--assembly" => Flag::Assembly,
                "-t" | "--tokens" => Flag::Tokens,
                _ => panic!("Incorrect usage"),
            },
            match args[i].as_str() {
                "-o" | "--output" => {
                    i += 1;
                    Some(args[i].clone())
                }
                _ => None,
            }, // TODO: Change this to an option.
        );

        i += 1;
    }

    dbg!(&flags);

    (filename, flags)
}
