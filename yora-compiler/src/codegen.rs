use crate::parser::*;

pub fn codegen(ast: Vec<Statement>) -> String {
    let mut assembly = String::new();

    assembly.push_str("global _start\n_start:\n");
    for statement in ast {
        let Statement::Exit(Expression::Literal(string)) = statement;

        assembly.push_str(&format!(
            "    mov rax, 60\n    mov rdi, {}\n    syscall\n",
            &string
        ));
    }

    assembly
}
