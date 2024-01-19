use std::fmt::Display;

use crate::{
    ir::{BinOp, Instruction, Operation, Terminator, ValueId},
    regalloc::{VReg, apply_alloc},
    vcode::{InstrSelector, LabelDest, VCodeGenerator, VCodeInstr},
};

pub const AARCH64_REGISTER_ZERO: usize = 0;
pub const AARCH64_REGISTER_X0  : usize = 1;
pub const AARCH64_REGISTER_X1  : usize = 2;
pub const AARCH64_REGISTER_X2  : usize = 3;
pub const AARCH64_REGISTER_X3  : usize = 4;
pub const AARCH64_REGISTER_X4  : usize = 5;
pub const AARCH64_REGISTER_X5  : usize = 6;
pub const AARCH64_REGISTER_X6  : usize = 7;
pub const AARCH64_REGISTER_X7  : usize = 8;
pub const AARCH64_REGISTER_X8  : usize = 9;
pub const AARCH64_REGISTER_X9  : usize = 10;
pub const AARCH64_REGISTER_X10 : usize = 11;
pub const AARCH64_REGISTER_X11 : usize = 12;
pub const AARCH64_REGISTER_X12 : usize = 13;
pub const AARCH64_REGISTER_X13 : usize = 14;
pub const AARCH64_REGISTER_X14 : usize = 15;
pub const AARCH64_REGISTER_X15 : usize = 16;
pub const AARCH64_REGISTER_IP0 : usize = 17;
pub const AARCH64_REGISTER_IP1 : usize = 18;
pub const AARCH64_REGISTER_X18 : usize = 19;
pub const AARCH64_REGISTER_X19 : usize = 20;
pub const AARCH64_REGISTER_X20 : usize = 21;
pub const AARCH64_REGISTER_X21 : usize = 22;
pub const AARCH64_REGISTER_X22 : usize = 23;
pub const AARCH64_REGISTER_X23 : usize = 24;
pub const AARCH64_REGISTER_X24 : usize = 25;
pub const AARCH64_REGISTER_X25 : usize = 26;
pub const AARCH64_REGISTER_X26 : usize = 27;
pub const AARCH64_REGISTER_X27 : usize = 28;
pub const AARCH64_REGISTER_X28 : usize = 29;
pub const AARCH64_REGISTER_FP  : usize = 30;
pub const AARCH64_REGISTER_LR  : usize = 31;
pub const AARCH64_REGISTER_SP  : usize = 32;

pub enum Aarch64Instr {
    PhiPlaceholder {
        dst: VReg,
        ops: Vec<VReg>,
    },
    AluOp {
        op: Aarch64AluOp,
        dst: VReg,
        src1: VReg,
        src2: VReg,
    },
    Msub {
        dst: VReg,
        src1: VReg,
        src2: VReg,
        src3: VReg,
    },
    B {
        dst: LabelDest,
    },
    Cbnz {
        src1: VReg,
        dst: LabelDest,
    },
    MovImm {
        dst: VReg,
        val: i64,
    },
    MovReg {
        dst: VReg,
        src: VReg,
    },
    Cal {
        dst: LabelDest,
    },
    Ret,
}

pub enum Aarch64AluOp {
    Add,
    Sub,
    Mul,
    Div,
    Lsl,
    Lsr,
    And,
    Orr,
    Eor,

    Udiv,
}

impl From<BinOp> for Aarch64AluOp {
    fn from(op: BinOp) -> Self {
        match op {
            BinOp::Add => Self::Add,
            BinOp::Sub => Self::Sub,
            BinOp::Mul => Self::Mul,
            BinOp::Div => Self::Div,
            BinOp::Shl => Self::Lsl,
            BinOp::Shr => Self::Lsr,
            BinOp::And => Self::And,
            BinOp::Or  => Self::Orr,

            _ => todo!()
        }
    }
}

impl Display for Aarch64AluOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add  => write!(f, "add"),
            Self::Sub  => write!(f, "sub"),
            Self::Mul  => write!(f, "mul"),
            Self::Div  => write!(f, "udiv"),
            Self::Lsl  => write!(f, "lsl"),
            Self::Lsr  => write!(f, "lsr"),
            Self::And  => write!(f, "and"),
            Self::Orr  => write!(f, "orr"),
            Self::Eor  => write!(f, "eor"),
            Self::Udiv => write!(f, "udiv"),
        }
    }
}

impl VCodeInstr for Aarch64Instr {
    fn get_usable_regs() -> &'static [VReg] {
        &[
            VReg::Real(AARCH64_REGISTER_X9),
            VReg::Real(AARCH64_REGISTER_X10),
            VReg::Real(AARCH64_REGISTER_X11),
            VReg::Real(AARCH64_REGISTER_X12),
            VReg::Real(AARCH64_REGISTER_X13),
            VReg::Real(AARCH64_REGISTER_X14),
            VReg::Real(AARCH64_REGISTER_X15),
            // IP0 is a temp used by some operations
            VReg::Real(AARCH64_REGISTER_IP1),
            VReg::Real(AARCH64_REGISTER_X18),
            VReg::Real(AARCH64_REGISTER_X19),
            VReg::Real(AARCH64_REGISTER_X20),
            VReg::Real(AARCH64_REGISTER_X21),
            VReg::Real(AARCH64_REGISTER_X22),
            VReg::Real(AARCH64_REGISTER_X23),
            VReg::Real(AARCH64_REGISTER_X24),
            VReg::Real(AARCH64_REGISTER_X25),
            VReg::Real(AARCH64_REGISTER_X26),
            VReg::Real(AARCH64_REGISTER_X27),
            VReg::Real(AARCH64_REGISTER_X28),
        ]
    }

    fn collect_registers(&self, regalloc: &mut impl crate::regalloc::Regalloc) {
        match self {
            Aarch64Instr::AluOp { dst, src1, src2, .. } => {
                regalloc.add_def(*dst);
                regalloc.add_use(*src1);
                regalloc.add_use(*src2);
            },
            Aarch64Instr::Msub { dst, src1, src2, src3 } => {
                regalloc.add_def(*dst);
                regalloc.add_use(*src1);
                regalloc.add_use(*src2);
                regalloc.add_use(*src3);
            },
            Aarch64Instr::MovImm { dst, .. } => regalloc.add_def(*dst),
            Aarch64Instr::Cbnz { src1, .. } => regalloc.add_use(*src1),
            Aarch64Instr::MovReg { dst, src } => {
                regalloc.add_def(*dst);
                regalloc.add_use(*src);
            },
            _ => {},
        }
    }

    fn apply_allocs(&mut self, allocs: &std::collections::HashMap<VReg, VReg>) {
        match self {
            Aarch64Instr::AluOp { dst, src1, src2, .. } => {
                apply_alloc(dst, allocs);
                apply_alloc(src1, allocs);
                apply_alloc(src2, allocs);
            },
            Aarch64Instr::Msub { dst, src1, src2, src3 } => {
                apply_alloc(dst, allocs);
                apply_alloc(src1, allocs);
                apply_alloc(src2, allocs);
                apply_alloc(src3, allocs);
            },
            Aarch64Instr::MovImm { dst, .. } => apply_alloc(dst, allocs),
            Aarch64Instr::Cbnz { src1, .. } => apply_alloc(src1, allocs),
            Aarch64Instr::MovReg { dst, src } => {
                apply_alloc(dst, allocs);
                apply_alloc(src, allocs);
            },
            _ => {},
        }
    }
}

impl Display for Aarch64Instr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Aarch64Instr::AluOp {
                op,
                dst,
                src1,
                src2,
            } => match op {
                _ => write!(
                    f,
                    "{op} {}, {}, {}",
                    format_vreg(dst),
                    format_vreg(src1),
                    format_vreg(src2)
                ),
            }
            Aarch64Instr::Msub {
                dst,
                src1,
                src2,
                src3,
            } => write!(
                f,
                "msub {}, {}, {}, {}",
                format_vreg(dst),
                format_vreg(src1),
                format_vreg(src2),
                format_vreg(src3)
            ),
            Aarch64Instr::B { dst } => write!(f, "b {dst}"),
            Aarch64Instr::MovImm { dst, val } => write!(
                f,
                "mov {}, {val}",
                format_vreg(dst)
            ),
            Aarch64Instr::Cbnz { src1, dst } => write!(
                f,
                "cbnz {}, {dst}",
                format_vreg(src1)
            ),
            Aarch64Instr::MovReg { dst, src } => write!(
                f,
                "mov {}, {}",
                format_vreg(dst),
                format_vreg(src)
            ),
            Aarch64Instr::Cal { dst } => write!(f, "bl {dst}"),
            Aarch64Instr::Ret => write!(f, "ret"),
            Aarch64Instr::PhiPlaceholder { dst, ops } => write!(
                f,
                "// phi {} {}",
                format_vreg(dst),
                ops.iter()
                    .map(|v| format_vreg(v))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
        }
    }
}

fn format_vreg(v: &VReg) -> String {
    match v {
        VReg::Virtual(v) => format!("v{v}"),
        VReg::Real(AARCH64_REGISTER_ZERO) => "xzr".to_string(),
        VReg::Real(AARCH64_REGISTER_SP) => "sp".to_string(),
        VReg::Real(r) => format!("x{}", r - 1),
        VReg::Spilled(s) => format!("s{s}"),
    }
}

#[derive(Default)]
pub struct Aarch64Selector;

impl InstrSelector for Aarch64Selector {
    type Instr = Aarch64Instr;
    fn select(&mut self, gen: &mut VCodeGenerator<Self::Instr>, instr: &Instruction) {
        let dst = if let Some(val) = instr.yielded {
            self.get_vreg(val)
        } else {
            VReg::Real(AARCH64_REGISTER_ZERO)
        };

        match &instr.operation {
            Operation::BinOp(op, lhs, rhs) => {
                let src1 = self.get_vreg(*lhs);
                let src2 = self.get_vreg(*rhs);
                match op {
                    BinOp::Mod => {
                        gen.push_instr(Aarch64Instr::AluOp {
                            op: Aarch64AluOp::Udiv,
                            dst: VReg::Real(AARCH64_REGISTER_IP0),
                            src1,
                            src2,
                        });
                        gen.push_instr(Aarch64Instr::Msub {
                            dst,
                            src1: VReg::Real(AARCH64_REGISTER_IP0),
                            src2,
                            src3: src1
                        });
                    }
                    _ => {
                        gen.push_instr(Aarch64Instr::AluOp {
                            op: (*op).into(),
                            dst,
                            src1,
                            src2,
                        });
                    }
                }
            }
            Operation::Integer(val) => {
                gen.push_instr(Aarch64Instr::MovImm { dst, val: *val });
            }
            Operation::LoadVar(_) | Operation::StoreVar(..) => unreachable!(), // THESE NEVER GET EXECUTED (removed in algos::lower_to_ssa::lower())
            Operation::Phi(vals) => {
                gen.push_instr(Aarch64Instr::PhiPlaceholder {
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
                gen.push_instr(Aarch64Instr::Cbnz {
                    src1: self.get_vreg(*val),
                    dst: LabelDest::Block(t.0),
                });
                gen.push_instr(Aarch64Instr::B {
                    dst: LabelDest::Block(f.0),
                });
            }
            Terminator::Jump(l) => {
                gen.push_instr(Aarch64Instr::B {
                    dst: LabelDest::Block(l.0),
                });
            }
            Terminator::Return(val) => {
                gen.push_instr(Aarch64Instr::MovReg {
                    dst: VReg::Real(AARCH64_REGISTER_X0),
                    src: self.get_vreg(*val),
                });
                gen.push_instr(Aarch64Instr::Ret);
            }
            _ => todo!(),
        }
    }
}

impl Aarch64Selector {
    #[inline]
    pub fn get_vreg(&self, val: ValueId) -> VReg {
        VReg::Virtual(val.0)
    }
}
