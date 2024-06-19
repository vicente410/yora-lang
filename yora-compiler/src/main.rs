use ir_generation::*;
use lexer::*;
use parser::*;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::process;
use std::process::Command;

mod ir_generation;
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
    Ast,
}

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let (filename, flags) = parse_args(&mut args);

    let source = fs::read_to_string(format!("{}.yr", filename)).unwrap();
    let assembly = compile(source, &flags);

    let (asm_name, obj_name, exec_name) = get_filenames(filename, &flags);

    let mut asm_file = File::create(&asm_name).unwrap();
    asm_file.write_all(&assembly.into_bytes()).unwrap();

    if flags.contains_key(&Flag::Assembly) {
        process::exit(0);
    }

    assemble(&asm_name, &obj_name);
    link(&obj_name, &exec_name);

    remove_tmpfiles(asm_name, obj_name, flags);
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
                        "ast" => DebugOptions::Ast,
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

fn compile(source: String, flags: &HashMap<Flag, Option<String>>) -> String {
    let tokens = lex(source);

    if flags.contains_key(&Flag::Debug(DebugOptions::Tokens)) {
        dbg!(&tokens);
        process::exit(0);
    }

    let ast = parse(tokens);

    if flags.contains_key(&Flag::Debug(DebugOptions::Ast)) {
        dbg!(&ast);
        process::exit(0);
    }

    codegen(ast)
}

fn get_filenames(
    filename: String,
    flags: &HashMap<Flag, Option<String>>,
) -> (String, String, String) {
    let asm_name = if flags.contains_key(&Flag::Assembly) {
        if flags.contains_key(&Flag::Output) {
            if let Some(string) = &flags[&Flag::Output] {
                string.clone()
            } else {
                panic!();
            }
        } else {
            format!("{}.asm", filename)
        }
    } else {
        get_tmpfile_path()
    };

    let obj_name = get_tmpfile_path();

    let exec_name = if flags.contains_key(&Flag::Output) {
        if let Some(string) = &flags[&Flag::Output] {
            string.clone()
        } else {
            panic!();
        }
    } else {
        filename
    };

    (asm_name, obj_name, exec_name)
}

fn assemble(asm_name: &String, obj_name: &String) {
    Command::new("nasm")
        .args(["-felf64", "-o", obj_name, asm_name])
        .output()
        .expect("Failed to execute \"nasm\"");
}

fn link(obj_name: &String, exec_name: &String) {
    Command::new("ld")
        .args(["-o", exec_name, obj_name])
        .output()
        .expect("Failed to execute \"ld\"");
}

fn get_tmpfile_path() -> String {
    String::from_utf8(
        Command::new("mktemp")
            .output()
            .expect("Failed to create temporary file.")
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string()
}

fn remove_tmpfiles(asm_name: String, obj_name: String, flags: HashMap<Flag, Option<String>>) {
    remove_file(obj_name);
    if !flags.contains_key(&Flag::Assembly) {
        remove_file(asm_name);
    }
}

fn remove_file(filename: String) {
    Command::new("rm")
        .args([filename])
        .output()
        .expect("Failed to remove file");
}
