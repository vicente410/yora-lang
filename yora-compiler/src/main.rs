use std::collections::HashMap;
use std::env;
use std::process;
use yora_compiler::*;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let mut compiler = Compiler::new();
    let (filename, flags) = parse_args(&mut args);

    compiler.set_filenames(&filename);
    compiler.set_flags(flags);
    compiler.compile();
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
            "-h" | "--help" => {
                print!(
                    "Usage: yora-compiler [options] file\n\
                        Option:\n\
                        \t-h, --help\t\tDisplay this message\n\
                        \t-o, --output <filename>\tWrite output to <filename>\n\
                        \t-d, --debug [tokens, ast]\tPrints debug information\n\
                        \t-s, --assembly\t\tCompile only; do not assemble or link\n"
                );
                process::exit(0);
            }
            _ => panic!("Incorrect usage"),
        };

        flags.insert(flag, value);

        i += 1;
    }

    (filename, flags)
}
