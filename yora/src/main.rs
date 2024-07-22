use std::env;
use std::process;
use yora::run;

fn main() {
    let args: Vec<String> = env::args().collect();

    let (filename, debug_ast) = parse_args(args);

    run(filename, debug_ast);
}

fn parse_args(args: Vec<String>) -> (String, bool) {
    let mut filename = String::new();
    let mut debug_ast = false;

    for arg in args {
        match arg.as_str() {
            "--ast" => debug_ast = true,
            _ => filename = arg,
        }
    }

    if filename == *"" {
        eprintln!("No filename given.");
        process::exit(1);
    }

    (filename, debug_ast)
}
