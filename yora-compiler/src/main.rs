use lexer::*;
use parser::*;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::process;
use std::process::Command;

mod lexer;
mod parser;

#[derive(Eq, Debug, Hash, PartialEq)]
enum Flag {
    Output,
    Debug(DebugOptions),
    Assembly,
}

#[derive(Eq, Debug, Hash, PartialEq)]
enum DebugOptions {
    Tokens,
    AST,
}

fn main() {
    let mut args: Vec<String> = env::args().collect();

    let (filename, flags) = parse_args(&mut args);

    let source = fs::read_to_string(&format!("{}.yr", filename)).unwrap();

    let ast = compile(source, &flags);

    output(ast, filename, flags);
}

fn parse_args(args: &mut Vec<String>) -> (String, HashMap<Flag, Option<String>>) {
    let mut filename = args.pop().unwrap();
    let mut i = 1;
    let mut flags: HashMap<Flag, Option<String>> = HashMap::new();

    match filename.strip_suffix(".yr") {
        Some(res) => filename = res.to_string(),
        None => panic!("Incorrect extension: please use .yr"),
    }

    while i < args.len() {
        let (flag, value) = match args[i].as_str() {
            "-o" | "--output" => {
                i += 1;
                (Flag::Output, Some(args[i].clone()))
            }
            "-d" | "--debug" => {
                i += 1;
                (
                    Flag::Debug(match args[i].as_str() {
                        "tokens" => DebugOptions::Tokens,
                        "ast" => DebugOptions::AST,
                        _ => panic!("Debug option not found."),
                    }),
                    None,
                )
            }
            "-s" | "--assembly" => (Flag::Assembly, None),
            _ => panic!("Incorrect usage"),
        };

        flags.insert(flag, value);

        i += 1;
    }

    (filename, flags)
}

fn compile(source: String, flags: &HashMap<Flag, Option<String>>) -> Vec<Statement> {
    let tokens = lex(source);

    if flags.contains_key(&Flag::Debug(DebugOptions::Tokens)) {
        dbg!(&tokens);
        process::exit(0);
    }

    let ast = parse(tokens);

    if flags.contains_key(&Flag::Debug(DebugOptions::AST)) {
        dbg!(&ast);
        process::exit(0);
    }

    ast
}

fn output(ast: Vec<Statement>, filename: String, flags: HashMap<Flag, Option<String>>) {
    let mut buffer = String::new();
    let mut output = filename.clone();

    if flags.contains_key(&Flag::Output) {
        if let Some(string) = &flags[&Flag::Output] {
            output = string.to_string();
        }
    }

    buffer.push_str("global _start\n_start:\n");

    for statement in ast {
        let Statement::Exit(Expression::Literal(string)) = statement;

        buffer.push_str(&format!(
            "    mov rax, 60\n    mov rdi, {}\n    syscall\n",
            &string
        ));
    }

    let mut file = File::create(format!("{}.asm", filename)).unwrap();
    file.write_all(&buffer.into_bytes()).unwrap();

    if flags.contains_key(&Flag::Assembly) {
        process::exit(0);
    }

    Command::new("nasm")
        .args(["-f elf64", &format!("{}.asm", filename)])
        .output()
        .expect("Failed to execute \"nasm\"");

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
