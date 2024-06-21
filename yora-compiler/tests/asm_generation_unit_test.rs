use yora_compiler::asm_generation::*;
use yora_compiler::ir_generation::Ir;

#[test]
fn test_asm_generation() {
    let input = vec![
        Ir::Assign("t1".to_string(), "2".to_string()),
        Ir::Assign("t2".to_string(), "3".to_string()),
        Ir::Add("t1".to_string(), "t2".to_string()),
        Ir::Exit("t1".to_string()),
    ];
    let output = "global _start\n\
                    _start:\n\
                        \tmov rbp, rsp\n\
                        \tmov rbx, 2\n\
                        \tmov r10, 3\n\
                        \tadd rbx, r10\n\n\
                        \tmov rdi, rbx\n\
                        \tmov rax, 60\n\
                        \tsyscall\n";
    assert_eq!(generate_asm(input), output);
}
