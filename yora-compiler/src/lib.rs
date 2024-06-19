use ir_generation::*;
use lexer::*;
use parser::*;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::process;
use std::process::Command;

mod ir_generation;
mod lexer;
mod parser;

#[derive(Eq, Debug, Hash, PartialEq)]
pub enum Flag {
    Output,
    Debug(DebugOptions),
    Assembly,
}

#[derive(Eq, Debug, Hash, PartialEq)]
pub enum DebugOptions {
    Tokens,
    Ast,
    Ir,
}

pub struct Compiler {
    filename: String,
    asm_name: String,
    obj_name: String,
    exec_name: String,
    flags: HashMap<Flag, Option<String>>,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            filename: String::new(),
            asm_name: String::new(),
            obj_name: String::new(),
            exec_name: String::new(),
            flags: HashMap::new(),
        }
    }

    pub fn set_flags(&mut self, flags: HashMap<Flag, Option<String>>) {
        self.flags = flags;
    }

    pub fn set_filename(&mut self, filename: &String) {
        self.filename = filename.to_string();
        self.asm_name = if self.flags.contains_key(&Flag::Assembly) {
            if self.flags.contains_key(&Flag::Output) {
                if let Some(string) = &self.flags[&Flag::Output] {
                    string.clone()
                } else {
                    panic!();
                }
            } else {
                format!("{}.asm", filename)
            }
        } else {
            Compiler::get_tmpfile_path()
        };
        self.obj_name = Compiler::get_tmpfile_path();
        self.exec_name = if self.flags.contains_key(&Flag::Output) {
            if let Some(string) = &self.flags[&Flag::Output] {
                string.clone()
            } else {
                panic!();
            }
        } else {
            filename.clone()
        };
    }

    pub fn compile(&self) {
        let source = fs::read_to_string(format!("{}.yr", self.filename)).unwrap();
        let assembly = self.get_assembly(source);

        let mut asm_file = File::create(&self.asm_name).unwrap();
        asm_file.write_all(&assembly.into_bytes()).unwrap();

        if self.flags.contains_key(&Flag::Assembly) {
            process::exit(0);
        }

        self.assemble();
        self.link();
        self.remove_tmpfiles();
    }

    fn get_assembly(&self, source: String) -> String {
        let tokens = lex(source);

        if self.flags.contains_key(&Flag::Debug(DebugOptions::Tokens)) {
            dbg!(&tokens);
            process::exit(0);
        }

        let ast = parse(tokens);

        if self.flags.contains_key(&Flag::Debug(DebugOptions::Ast)) {
            dbg!(&ast);
            process::exit(0);
        }

        let ir = generate_ir(ast);

        if self.flags.contains_key(&Flag::Debug(DebugOptions::Ir)) {
            dbg!(&ir);
            process::exit(0);
        }

        "".to_string()
    }

    fn assemble(&self) {
        Command::new("nasm")
            .args(["-felf64", "-o", &self.obj_name, &self.asm_name])
            .output()
            .expect("Failed to execute \"nasm\"");
    }

    fn link(&self) {
        Command::new("ld")
            .args(["-o", &self.exec_name, &self.obj_name])
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

    fn remove_tmpfiles(&self) {
        Compiler::remove_file(&self.obj_name);
        if !self.flags.contains_key(&Flag::Assembly) {
            Compiler::remove_file(&self.asm_name);
        }
    }

    fn remove_file(filename: &String) {
        Command::new("rm")
            .args([filename])
            .output()
            .expect("Failed to remove file");
    }
}
