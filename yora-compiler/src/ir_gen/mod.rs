use std::fmt;

use crate::parser::*;

#[derive(Debug, PartialEq, Clone)]
pub enum Ir {
    Op { dest: String, src: String, op: Op },
    Exit { src: String },
    Set { dest: String, cond: Cond },
    Label(String),
    Jmp { label: String },
    JmpCond { src: String, label: String },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Op {
    Assign,
    Cmp,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Not,
    And,
    Or,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Cond {
    Eq,
    Neq,
    Lt,
    Leq,
    Gt,
    Geq,
}

struct Nums {
    tmp: u32,
    ifs: u32,
    loops: u32,
}

impl fmt::Display for Ir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ir::Op { dest, src, op } => write!(f, "{} {} {}", dest, op, src),
            Ir::Exit { src } => write!(f, "exit {}", src),
            Ir::Set { dest, cond } => write!(f, "set{} {}", cond, dest),
            Ir::Label(str) => write!(f, "{}:", str),
            Ir::Jmp { label } => write!(f, "jmp {}", label),
            Ir::JmpCond { src, label } => write!(f, "if {} jmp {}", src, label),
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Op::Assign => ":=",
                Op::Add => "+=",
                Op::Sub => "-=",
                Op::Mul => "*=",
                Op::Div => "/=",
                Op::Mod => "%=",
                Op::Not => "!=",
                Op::And => "&=",
                Op::Or => "|=",
                Op::Cmp => "cmp",
            }
        )
    }
}

impl fmt::Display for Cond {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Cond::Eq => "e",
                Cond::Neq => "ne",
                Cond::Lt => "l",
                Cond::Leq => "le",
                Cond::Gt => "g",
                Cond::Geq => "ge",
            }
        )
    }
}

pub fn generate_ir(ast: Vec<Expression>) -> Vec<Ir> {
    let mut inter_repr = Vec::new();
    let mut tmp_vec = Vec::new();
    let mut nums = Nums {
        tmp: 0,
        ifs: 0,
        loops: 0,
    };

    for expr in ast {
        get_value(&expr, &mut tmp_vec, &mut nums);
        inter_repr.append(&mut tmp_vec);
        tmp_vec.clear();
    }

    inter_repr
}

fn get_value(expr: &Expression, tmp_vec: &mut Vec<Ir>, nums: &mut Nums) -> String {
    match &expr.kind {
        ExpressionKind::Exit(val) => {
            let arg = get_value(val, tmp_vec, nums);
            tmp_vec.push(Ir::Exit { src: arg.clone() });
            arg
        }
        ExpressionKind::Assign(ref dest, ref src) | ExpressionKind::Declare(ref dest, ref src) => {
            let arg1 = get_value(dest, tmp_vec, nums);
            let arg2 = get_value(src, tmp_vec, nums);
            tmp_vec.push(Ir::Op {
                dest: arg1.clone(),
                src: arg2,
                op: Op::Assign,
            });
            arg1
        }
        ExpressionKind::Add(ref dest, ref src)
        | ExpressionKind::Sub(ref dest, ref src)
        | ExpressionKind::Mul(ref dest, ref src)
        | ExpressionKind::Div(ref dest, ref src)
        | ExpressionKind::Mod(ref dest, ref src)
        | ExpressionKind::And(ref dest, ref src)
        | ExpressionKind::Or(ref dest, ref src) => {
            nums.tmp += 1;
            let destination = format!("t{}", nums.tmp);

            let arg1 = get_value(dest, tmp_vec, nums);
            let arg2 = get_value(src, tmp_vec, nums);

            tmp_vec.push(Ir::Op {
                dest: destination.clone(),
                src: arg2.clone(),
                op: Op::Assign,
            });

            tmp_vec.push(get_operation(expr, destination.clone(), arg1));
            destination
        }
        ExpressionKind::Eq(ref dest, ref src)
        | ExpressionKind::Neq(ref dest, ref src)
        | ExpressionKind::Lt(ref dest, ref src)
        | ExpressionKind::Leq(ref dest, ref src)
        | ExpressionKind::Gt(ref dest, ref src)
        | ExpressionKind::Geq(ref dest, ref src) => {
            let arg1 = get_value(dest, tmp_vec, nums);
            let arg2 = get_value(src, tmp_vec, nums);
            tmp_vec.push(Ir::Op {
                dest: arg1.clone(),
                src: arg2,
                op: Op::Cmp,
            });
            nums.tmp += 1;
            tmp_vec.push(Ir::Set {
                dest: format!("t{}", nums.tmp),
                cond: get_condition(expr),
            });
            format!("t{}", nums.tmp)
        }
        ExpressionKind::IntLit(int) => {
            nums.tmp += 1;
            tmp_vec.push(Ir::Op {
                dest: format!("t{}", nums.tmp),
                src: int.to_string(),
                op: Op::Assign,
            });
            format!("t{}", nums.tmp)
        }
        ExpressionKind::BoolLit(bool) => {
            nums.tmp += 1;
            tmp_vec.push(Ir::Op {
                dest: format!("t{}", nums.tmp),
                src: if bool == "true" {
                    "1".to_string()
                } else {
                    "0".to_string()
                },
                op: Op::Assign,
            });
            format!("t{}", nums.tmp)
        }
        ExpressionKind::Identifier(id) => id.to_string(),
        ExpressionKind::If(cond, seq) => {
            // todo: remove current_ifs
            nums.ifs += 1;
            let current_ifs = nums.ifs;
            let src = get_value(cond, tmp_vec, nums);
            tmp_vec.push(Ir::JmpCond {
                src,
                label: format!("end_if_{}", current_ifs),
            });
            let seq_value = get_value(seq, tmp_vec, nums);
            tmp_vec.push(Ir::Label(format!("end_if_{}", current_ifs)));
            seq_value
        }
        ExpressionKind::IfElse(cond, if_seq, else_seq) => {
            nums.ifs += 1;
            let current_ifs = nums.ifs;
            let src = get_value(cond, tmp_vec, nums);
            tmp_vec.push(Ir::JmpCond {
                src,
                label: format!("else_{}", current_ifs),
            });
            let if_seq_value = get_value(if_seq, tmp_vec, nums);
            tmp_vec.push(Ir::Jmp {
                label: format!("end_if_{}", current_ifs),
            });
            tmp_vec.push(Ir::Label(format!("else_{}", current_ifs)));
            get_value(else_seq, tmp_vec, nums);
            tmp_vec.push(Ir::Label(format!("end_if_{}", current_ifs)));
            if_seq_value
        }
        ExpressionKind::Loop(seq) => {
            tmp_vec.push(Ir::Label(format!("loop_{}", nums.loops)));
            let seq_value = get_value(seq, tmp_vec, nums);
            tmp_vec.push(Ir::Jmp {
                label: format!("loop_{}", nums.loops),
            });
            tmp_vec.push(Ir::Label(format!("loop_end_{}", nums.loops)));
            nums.loops += 1;
            seq_value
        }
        ExpressionKind::Break => {
            tmp_vec.push(Ir::Jmp {
                label: format!("loop_end_{}", nums.loops),
            });
            "".to_string()
        }
        ExpressionKind::Continue => {
            tmp_vec.push(Ir::Jmp {
                label: format!("loop_{}", nums.loops),
            });
            "".to_string()
        }
        ExpressionKind::Sequence(seq) => {
            for expr in &seq[0..seq.len() - 1] {
                get_value(expr, tmp_vec, nums);
            }
            get_value(&seq[seq.len() - 1], tmp_vec, nums)
        }
        ExpressionKind::Not(arg) => {
            nums.tmp += 1;
            let arg1 = get_value(arg, tmp_vec, nums);
            tmp_vec.push(Ir::Op {
                dest: format!("t{}", nums.tmp),
                src: arg1.clone(),
                op: Op::Not,
            });
            format!("t{}", nums.tmp)
        }
    }
}

fn get_operation(operation: &Expression, arg1: String, arg2: String) -> Ir {
    Ir::Op {
        dest: arg1,
        src: arg2,
        op: match operation.kind {
            ExpressionKind::Add(..) => Op::Add,
            ExpressionKind::Sub(..) => Op::Sub,
            ExpressionKind::Mul(..) => Op::Mul,
            ExpressionKind::Div(..) => Op::Div,
            ExpressionKind::Mod(..) => Op::Mod,
            ExpressionKind::And(..) => Op::And,
            ExpressionKind::Or(..) => Op::Or,
            _ => panic!("Unexpected operation"),
        },
    }
}

fn get_condition(expr: &Expression) -> Cond {
    match &expr.kind {
        ExpressionKind::Eq(..) => Cond::Eq,
        ExpressionKind::Neq(..) => Cond::Neq,
        ExpressionKind::Lt(..) => Cond::Lt,
        ExpressionKind::Leq(..) => Cond::Leq,
        ExpressionKind::Gt(..) => Cond::Gt,
        ExpressionKind::Geq(..) => Cond::Geq,
        _ => panic!("Invalid condition"),
    }
}
