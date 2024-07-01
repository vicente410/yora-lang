use analyzer::analyze;
use asm_gen::*;
use ir_gen::*;
use lexer::*;
use optimizer::optimize;
use parser::*;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::process;
use std::process::Command;

pub mod analyzer;
pub mod asm_gen;
pub mod ir_gen;
pub mod lexer;
pub mod optimizer;
pub mod parser;

#[derive(Eq, Debug, Hash, PartialEq)]
pub enum Flag {
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
    flags: Vec<Flag>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            filename: String::new(),
            asm_name: String::new(),
            obj_name: String::new(),
            exec_name: String::new(),
            flags: Vec::new(),
        }
    }

    pub fn set_flags(&mut self, flags: Vec<Flag>) {
        self.flags = flags;
    }

    pub fn set_filename(&mut self, filename: &String, exec_name: &String) {
        self.filename = filename.to_string();
        self.obj_name = Compiler::get_tmpfile_path();
        self.exec_name = if !exec_name.is_empty() {
            exec_name.to_string()
        } else {
            filename.clone()
        };

        let asm_name = self.exec_name.clone();

        self.asm_name = if self.flags.contains(&Flag::Assembly) {
            format!("{}.asm", asm_name)
        } else {
            Compiler::get_tmpfile_path()
        };
    }

    pub fn compile(&self) {
        let source = fs::read_to_string(format!("{}.yr", self.filename)).unwrap();
        let assembly = self.get_assembly(source);

        let mut asm_file = File::create(&self.asm_name).unwrap();
        asm_file.write_all(&assembly.into_bytes()).unwrap();

        if self.flags.contains(&Flag::Assembly) {
            process::exit(0);
        }

        self.assemble();
        self.link();
        self.remove_tmpfiles();
    }

    fn get_assembly(&self, source: String) -> String {
        let tokens = lex(source);

        if self.flags.contains(&Flag::Debug(DebugOptions::Tokens)) {
            for token in &tokens {
                println!("{}", token);
            }
            process::exit(0);
        }

        let ast = parse(tokens);

        if self.flags.contains(&Flag::Debug(DebugOptions::Ast)) {
            for expr in &ast {
                println!("{}", expr);
            }
            process::exit(0);
        }

        analyze(&ast);

        let ir = optimize(generate_ir(ast));

        if self.flags.contains(&Flag::Debug(DebugOptions::Ir)) {
            dbg!(&ir);
            process::exit(0);
        }

        generate_asm(ir)
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
        if !self.flags.contains(&Flag::Assembly) {
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
