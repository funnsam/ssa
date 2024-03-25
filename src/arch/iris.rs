//! # Iris default calling convention:
//! - `R1`: return value
//! - `R2` - `R8`: arguments
//! - `R9` - `R25`: scratch register / caller save

use std::fmt::Display;

use crate::{
    ir::{BinOp, Instruction, Operation, Terminator, ValueId},
    regalloc::{apply_alloc, VReg},
    vcode::{InstrSelector, LabelDest, VCodeGenerator, VCodeInstr},
};

pub const IRIS_REG_ZR: usize = 0;
pub const IRIS_REG_1: usize = 1;
pub const IRIS_REG_2: usize = 2;
pub const IRIS_REG_3: usize = 3;
pub const IRIS_REG_4: usize = 4;
pub const IRIS_REG_5: usize = 5;
pub const IRIS_REG_6: usize = 6;
pub const IRIS_REG_7: usize = 7;
pub const IRIS_REG_8: usize = 8;
pub const IRIS_REG_9: usize = 9;
pub const IRIS_REG_10: usize = 10;
pub const IRIS_REG_11: usize = 11;
pub const IRIS_REG_12: usize = 12;
pub const IRIS_REG_13: usize = 13;
pub const IRIS_REG_14: usize = 14;
pub const IRIS_REG_15: usize = 15;
pub const IRIS_REG_16: usize = 16;
pub const IRIS_REG_17: usize = 17;
pub const IRIS_REG_18: usize = 18;
pub const IRIS_REG_19: usize = 19;
pub const IRIS_REG_20: usize = 20;
pub const IRIS_REG_21: usize = 21;
pub const IRIS_REG_22: usize = 22;
pub const IRIS_REG_23: usize = 23;
pub const IRIS_REG_24: usize = 24;
pub const IRIS_REG_25: usize = 25;
pub const IRIS_REG_26: usize = 26;
pub const IRIS_REG_27: usize = 27;
pub const IRIS_REG_28: usize = 28;
pub const IRIS_REG_29: usize = 29;


pub enum IrisInstr {
    PhiPlaceholder {
        dst: VReg,
        ops: Vec<VReg>,
    },
    AluOp {
        op: IrisAluOp,
        dst: VReg,
        src1: VReg,
        src2: VReg,
    },
    Jmp {
        dst: LabelDest,
    },
    Beq {
        src1: VReg,
        dst: LabelDest,
    },
    Imm {
        dst: VReg,
        val: i64,
    },
    Mov {
        dst: VReg,
        src: VReg,
    },
    Cal {
        dst: LabelDest,
    },
    Ret,
}

pub enum IrisAluOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Xor,
    Not,
    Neg,
    Rsh,
    Lsh,
    Ssete,
    Ssetne,
    Ssetl,
    Ssetle,
    Ssetg,
    Ssetge,
}

impl From<BinOp> for IrisAluOp {
    fn from(op: BinOp) -> Self {
        match op {
            BinOp::Add => IrisAluOp::Add,
            BinOp::Sub => IrisAluOp::Sub,
            BinOp::Mul => IrisAluOp::Mul,
            BinOp::Div => IrisAluOp::Div,
            BinOp::Mod => IrisAluOp::Mod,
            BinOp::And => IrisAluOp::And,
            BinOp::Or => IrisAluOp::Or,
            BinOp::Xor => IrisAluOp::Xor,
            BinOp::Eq => IrisAluOp::Ssete,
            BinOp::Ne => IrisAluOp::Ssetne,
            BinOp::Lt => IrisAluOp::Ssetl,
            BinOp::Le => IrisAluOp::Ssetle,
            BinOp::Gt => IrisAluOp::Ssetg,
            BinOp::Ge => IrisAluOp::Ssetge,
            BinOp::Shl => IrisAluOp::Lsh,
            BinOp::Shr => IrisAluOp::Rsh,
        }
    }
}

impl VCodeInstr for IrisInstr {
    fn get_usable_regs() -> &'static [VReg] {
        &[
            VReg::Real(IRIS_REG_1),
            VReg::Real(IRIS_REG_2),
            VReg::Real(IRIS_REG_3),
            VReg::Real(IRIS_REG_4),
            VReg::Real(IRIS_REG_5),
            VReg::Real(IRIS_REG_6),
            VReg::Real(IRIS_REG_7),
            VReg::Real(IRIS_REG_8),
            VReg::Real(IRIS_REG_9),
            VReg::Real(IRIS_REG_10),
            VReg::Real(IRIS_REG_11),
            VReg::Real(IRIS_REG_12),
            VReg::Real(IRIS_REG_13),
            VReg::Real(IRIS_REG_14),
            VReg::Real(IRIS_REG_15),
            VReg::Real(IRIS_REG_16),
            VReg::Real(IRIS_REG_17),
            VReg::Real(IRIS_REG_18),
            VReg::Real(IRIS_REG_19),
            VReg::Real(IRIS_REG_20),
            VReg::Real(IRIS_REG_21),
            VReg::Real(IRIS_REG_22),
            VReg::Real(IRIS_REG_23),
            VReg::Real(IRIS_REG_24),
            VReg::Real(IRIS_REG_25),
            VReg::Real(IRIS_REG_26),
            VReg::Real(IRIS_REG_27),
            VReg::Real(IRIS_REG_28),
            VReg::Real(IRIS_REG_29),
        ]
    }

    fn collect_registers(&self, regalloc: &mut impl crate::regalloc::Regalloc) {
        match self {
            Self::AluOp {
                dst, src1, src2, ..
            } => {
                regalloc.add_def(*dst);
                regalloc.add_use(*src1);
                regalloc.add_use(*src2);
            }
            Self::Jmp { .. } => (),
            Self::Beq { src1, .. } => {
                regalloc.add_use(*src1);
            }
            Self::Imm { dst, .. } => {
                regalloc.add_def(*dst);
            }
            Self::Mov { dst, src } => {
                regalloc.add_def(*dst);
                regalloc.add_use(*src);
                regalloc.coalesce_move(*src, *dst);
            }
            Self::PhiPlaceholder { dst, ops } => {
                regalloc.add_def(*dst);
                for i in ops.iter() {
                    regalloc.add_use(*i);
                    regalloc.coalesce_move(*i, *dst);
                }
            }
            _ => (),
        }
    }

    fn apply_allocs(&mut self, allocs: &std::collections::HashMap<VReg, VReg>) {
        match self {
            Self::AluOp {
                dst, src1, src2, ..
            } => {
                apply_alloc(dst, allocs);
                apply_alloc(src1, allocs);
                apply_alloc(src2, allocs);
            }
            Self::Jmp { .. } => (),
            Self::Beq { src1, .. } => {
                apply_alloc(src1, allocs);
            }
            Self::Imm { dst, .. } => {
                apply_alloc(dst, allocs);
            }
            Self::Mov { dst, src } => {
                apply_alloc(dst, allocs);
                apply_alloc(src, allocs);
            }
            Self::PhiPlaceholder { dst, ops } => {
                apply_alloc(dst, allocs);
                for i in ops.iter_mut() {
                    apply_alloc(i, allocs);
                }
            }
            _ => (),
        }
    }
}

impl Display for IrisInstr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IrisInstr::AluOp {
                op,
                dst,
                src1,
                src2,
            } => {
                write!(f, "{} {} {} {}", op, dst, src1, src2)
            }
            IrisInstr::Jmp { dst } => write!(f, "jmp {}", dst),
            IrisInstr::Imm { dst, val } => write!(f, "imm {} {}", dst, val),
            IrisInstr::Beq { src1, dst } => write!(f, "bnz {} {}", dst, src1),
            IrisInstr::Mov { dst, src } => write!(f, "mov {} {}", dst, src),
            IrisInstr::Cal { dst } => write!(f, "cal {}", dst),
            IrisInstr::Ret => write!(f, "ret"),
            IrisInstr::PhiPlaceholder { dst, ops } => write!(
                f,
                "phi {} {}",
                dst,
                ops.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
        }
    }
}

impl Display for IrisAluOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IrisAluOp::Add => write!(f, "add"),
            IrisAluOp::Sub => write!(f, "sub"),
            IrisAluOp::Mul => write!(f, "mul"),
            IrisAluOp::Div => write!(f, "div"),
            IrisAluOp::Mod => write!(f, "mod"),
            IrisAluOp::And => write!(f, "and"),
            IrisAluOp::Or => write!(f, "or"),
            IrisAluOp::Xor => write!(f, "xor"),
            IrisAluOp::Not => write!(f, "not"),
            IrisAluOp::Neg => write!(f, "neg"),
            IrisAluOp::Rsh => write!(f, "rsh"),
            IrisAluOp::Lsh => write!(f, "lsh"),
            IrisAluOp::Ssete => write!(f, "ssete"),
            IrisAluOp::Ssetne => write!(f, "ssetne"),
            IrisAluOp::Ssetl => write!(f, "ssetl"),
            IrisAluOp::Ssetle => write!(f, "ssetle"),
            IrisAluOp::Ssetg => write!(f, "ssetg"),
            IrisAluOp::Ssetge => write!(f, "ssetge"),
        }
    }
}

#[derive(Default)]
pub struct IrisSelector;

impl InstrSelector for IrisSelector {
    type Instr = IrisInstr;
    fn select(&mut self, gen: &mut VCodeGenerator<Self::Instr>, instr: &Instruction) {
        let dst = if let Some(val) = instr.yielded {
            self.get_vreg(val)
        } else {
            VReg::Real(IRIS_REG_ZR)
        };

        match &instr.operation {
            Operation::BinOp(op, lhs, rhs) => {
                let src1 = self.get_vreg(*lhs);
                let src2 = self.get_vreg(*rhs);
                gen.push_instr(IrisInstr::AluOp {
                    op: (*op).into(),
                    dst,
                    src1,
                    src2,
                });
            }
            Operation::Integer(val) => {
                gen.push_instr(IrisInstr::Imm { dst, val: *val });
            }
            Operation::LoadVar(_) | Operation::StoreVar(..) => (), // THESE NEVER GET EXECUTED (removed in algos::lower_to_ssa::lower())
            Operation::Phi(vals) => {
                gen.push_instr(IrisInstr::PhiPlaceholder {
                    dst,
                    ops: vals.iter().map(|v| self.get_vreg(*v)).collect(),
                });
            }
            _ => todo!(),
        }
    }

    fn select_terminator(&mut self, gen: &mut VCodeGenerator<Self::Instr>, term: &Terminator) {
        match term {
            Terminator::Branch(val, t, f) => {
                gen.push_instr(IrisInstr::Beq {
                    src1: self.get_vreg(*val),
                    dst: LabelDest::Block(t.0),
                });
                gen.push_instr(IrisInstr::Jmp {
                    dst: LabelDest::Block(f.0),
                });
            }
            Terminator::Jump(l) => {
                gen.push_instr(IrisInstr::Jmp {
                    dst: LabelDest::Block(l.0),
                });
            }
            Terminator::Return(val) => {
                gen.push_instr(IrisInstr::Mov {
                    dst: VReg::Real(IRIS_REG_1),
                    src: self.get_vreg(*val),
                });
                gen.push_instr(IrisInstr::Ret);
            }
            _ => todo!(),
        }
    }

    fn get_post_function_instructions(&mut self, gen: &mut VCodeGenerator<Self::Instr>) {
        
    }

    fn get_pre_function_instructions(&mut self, gen: &mut VCodeGenerator<Self::Instr>) {
        
    }
}

impl IrisSelector {
    #[inline]
    pub fn get_vreg(&self, val: ValueId) -> VReg {
        VReg::Virtual(val.0)
    }
}
