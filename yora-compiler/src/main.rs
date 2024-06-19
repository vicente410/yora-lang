use std::collections::HashMap;
use std::env;
use std::process;
use yora_compiler::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut compiler = Compiler::new();
    let (filename, flags) = parse_args(args);

    compiler.set_flags(flags);
    compiler.set_filename(&filename);
    compiler.compile();
}

fn parse_args(args: Vec<String>) -> (String, HashMap<Flag, Option<String>>) {
    let mut filename = String::new();
    let mut flags: HashMap<Flag, Option<String>> = HashMap::new();
    let mut args_it = args.iter().peekable();

    args_it.next();
    while let Some(arg) = args_it.next() {
        match arg.as_str() {
            "-o" | "--output" => {
                if let Some(arg) = args_it.next() {
                    flags.insert(Flag::Output, Some(arg.clone()));
                } else {
                    println!(
                        "Incorrect usage of output flag.\n\
                            Correct usage: \n\
                            \t-o, --output <filename>\tWrite output to <filename>"
                    );
                    process::exit(1);
                }
            }

            "-d" | "--debug" => {
                if let Some(arg) = args_it.next() {
                    flags.insert(
                        Flag::Debug(match arg.as_str() {
                            "tokens" => DebugOptions::Tokens,
                            "ast" => DebugOptions::Ast,
                            "ir" => DebugOptions::Ir,
                            _ => {
                                println!("Invalid debug option.");
                                process::exit(1);
                            }
                        }),
                        None,
                    );
                } else {
                    println!(
                        "Incorrect usage of debug flag.\n\
                            Correct usage:\n\
                            \t-d, --debug [tokens, ast]\tPrints debug information"
                    );
                    process::exit(1);
                }
            }

            "-s" | "--assembly" => {
                flags.insert(Flag::Assembly, None);
            }

            "-h" | "--help" => {
                print!(
                    "Usage: yora-compiler [options] file\n\
                        Option:\n\
                        \t-h, --help\t\t\tDisplay this message\n\
                        \t-o, --output <filename>\t\tWrite output to <filename>\n\
                        \t-d, --debug [tokens, ast, ir]\tPrints debug information\n\
                        \t-s, --assembly\t\t\tCompile only; do not assemble or link\n"
                );
                process::exit(0);
            }

            _ => {
                if arg[0..1] == *"-" {
                    println!("Incorrect usage");
                } else if filename == *"" {
                    filename = arg.clone();
                    match filename.strip_suffix(".yr") {
                        Some(res) => filename = res.to_string(),
                        None => {
                            println!("Incorrect extension: please use .yr");
                            process::exit(1);
                        }
                    };
                } else {
                    println!("More than one filename given.\nUse -h for help.");
                    process::exit(1);
                }
            }
        };
    }

    if filename == *"" {
        println!("No filename given.");
        process::exit(1);
    }

    (filename, flags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_system_output() {
        let input = vec![
            "program".to_string(),
            "test.yr".to_string(),
            "-o".to_string(),
            "output_file".to_string(),
        ];
        let mut output: HashMap<Flag, Option<String>> = HashMap::new();
        output.insert(Flag::Output, Some("output_file".to_string()));
        assert_eq!(parse_args(input), ("test".to_string(), output));
    }

    #[test]
    fn test_flag_system_debug() {
        let input = vec![
            "program".to_string(),
            "test.yr".to_string(),
            "-d".to_string(),
            "ast".to_string(),
        ];
        let mut output: HashMap<Flag, Option<String>> = HashMap::new();
        output.insert(Flag::Debug(DebugOptions::Ast), None);
        assert_eq!(parse_args(input), ("test".to_string(), output));
    }

    #[test]
    fn test_flag_system_asm() {
        let input = vec![
            "program".to_string(),
            "test.yr".to_string(),
            "-s".to_string(),
        ];
        let mut output: HashMap<Flag, Option<String>> = HashMap::new();
        output.insert(Flag::Assembly, None);
        assert_eq!(parse_args(input), ("test".to_string(), output));
    }
}
