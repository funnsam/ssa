use std::{collections::HashMap, fmt::Display};

use crate::{
    ir::{BinOp, Instruction, Operation, Terminator, ValueId, Function, Linkage},
    regalloc::{VReg, apply_alloc},
    vcode::{InstrSelector, LabelDest, VCode, VCodeGenerator, VCodeInstr, DisplayVCode},
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

pub const AARCH64_CALLEE: &'static [usize] = &[
    AARCH64_REGISTER_FP,
    AARCH64_REGISTER_LR,
    AARCH64_REGISTER_X19,
    AARCH64_REGISTER_X20,
    AARCH64_REGISTER_X21,
    AARCH64_REGISTER_X22,
    AARCH64_REGISTER_X23,
    AARCH64_REGISTER_X24,
    AARCH64_REGISTER_X25,
    AARCH64_REGISTER_X26,
    AARCH64_REGISTER_X27,
    AARCH64_REGISTER_X28,
];

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
    AluImm {
        op: Aarch64AluOp,
        dst: VReg,
        src1: VReg,
        src2: i64,
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
    Bl {
        dst: LabelDest,
    },
    Ret,
    LoadSp {
        dst: VReg,
        offset: i64
    },
    StoreSp {
        src: VReg,
        offset: i64
    },
    Autibsp
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
            // VReg::Real(AARCH64_REGISTER_IP0),
            // VReg::Real(AARCH64_REGISTER_IP1),
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
            Aarch64Instr::AluImm { dst, src1, .. } => {
                regalloc.add_def(*dst);
                regalloc.add_use(*src1);
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
            Aarch64Instr::LoadSp { dst, .. } => {
                regalloc.add_def(*dst);
            },
            Aarch64Instr::StoreSp { src, .. } => {
                regalloc.add_use(*src);
            },
            Aarch64Instr::B { .. }
                | Aarch64Instr::Bl { .. }
                | Aarch64Instr::PhiPlaceholder { .. }
                | Aarch64Instr::Ret
                | Aarch64Instr::Autibsp
            => {},
            // _ => {},
        }
    }

    fn apply_allocs(&mut self, allocs: &std::collections::HashMap<VReg, VReg>) {
        match self {
            Aarch64Instr::AluOp { dst, src1, src2, .. } => {
                apply_alloc(dst, allocs);
                apply_alloc(src1, allocs);
                apply_alloc(src2, allocs);
            },
            Aarch64Instr::AluImm { dst, src1, .. } => {
                apply_alloc(dst, allocs);
                apply_alloc(src1, allocs);
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
            Aarch64Instr::LoadSp { dst, .. } => apply_alloc(dst, allocs),
            Aarch64Instr::StoreSp { src, .. } => apply_alloc(src, allocs),
            Aarch64Instr::B { .. }
                | Aarch64Instr::Bl { .. }
                | Aarch64Instr::PhiPlaceholder { .. }
                | Aarch64Instr::Ret
                | Aarch64Instr::Autibsp
            => {},
        }
    }
}

impl DisplayVCode<Self> for Aarch64Instr {
    fn fmt_inst(&self, f: &mut std::fmt::Formatter<'_>, vcode: &VCode<Self>) -> std::fmt::Result {
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
            Aarch64Instr::AluImm {
                op,
                dst,
                src1,
                src2,
            } => match op {
                _ => write!(
                    f,
                    "{op} {}, {}, #{src2}",
                    format_vreg(dst),
                    format_vreg(src1),
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
            Aarch64Instr::B { dst } => write!(f, "b {}", dst.to_string(vcode)),
            Aarch64Instr::MovImm { dst, val } => write!(
                f,
                "mov {}, {val}",
                format_vreg(dst)
            ),
            Aarch64Instr::Cbnz { src1, dst } => write!(
                f,
                "cbnz {}, {}",
                format_vreg(src1),
                dst.to_string(vcode)
            ),
            Aarch64Instr::MovReg { dst, src } => write!(
                f,
                "mov {}, {}",
                format_vreg(dst),
                format_vreg(src)
            ),
            Aarch64Instr::Bl { dst } => write!(f, "bl {}", dst.to_string(vcode)),
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
            Aarch64Instr::LoadSp { dst: src, offset } => write!(
                f,
                "ldr {}, [sp, #{offset}]",
                format_vreg(src)
            ),
            Aarch64Instr::StoreSp { src, offset } => write!(
                f,
                "str {}, [sp, #{offset}]",
                format_vreg(src)
            ),
            Aarch64Instr::Autibsp => write!(f, "autibsp"),
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
pub struct Aarch64Selector {
    virtual_map: HashMap<usize, VReg>
}

impl InstrSelector for Aarch64Selector {
    type Instr = Aarch64Instr;
    fn select(
        &mut self,
        gen: &mut VCodeGenerator<Self::Instr>,
        instr: &Instruction,
        func: &Function
    ) {
        let dst = if let Some(val) = instr.yielded {
            self.get_vreg(val, gen)
        } else {
            VReg::Real(AARCH64_REGISTER_ZERO)
        };

        match &instr.operation {
            Operation::BinOp(op, lhs, rhs) => {
                let src1 = self.get_vreg(*lhs, gen);
                let src2 = self.get_vreg(*rhs, gen);
                match op {
                    BinOp::Mod => {
                        let tmp = gen.push_vreg();
                        gen.push_instr(Aarch64Instr::AluOp {
                            op: Aarch64AluOp::Udiv,
                            dst: tmp,
                            src1,
                            src2,
                        });
                        gen.push_instr(Aarch64Instr::Msub {
                            dst,
                            src1: tmp,
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
            },
            Operation::Integer(val) => {
                gen.push_instr(Aarch64Instr::MovImm { dst, val: *val });
            },
            Operation::LoadVar(_) | Operation::StoreVar(..) => unreachable!(), // THESE NEVER GET EXECUTED (removed in algos::lower_to_ssa::lower())
            Operation::Call(func, args) => {
                // TODO: save x0-x7 + x9-x15

                for (i, a) in args.iter().enumerate() {
                    if i > 7 {
                        todo!();
                    }

                    let src = self.get_vreg(*a, gen);
                    gen.push_instr(Aarch64Instr::MovReg {
                        dst: VReg::Real(AARCH64_REGISTER_X0 + i),
                        src
                    });
                }

                gen.push_instr(Aarch64Instr::Bl { dst: LabelDest::Function(func.0) });
                gen.push_instr(Aarch64Instr::MovReg {
                    dst,
                    src: VReg::Real(AARCH64_REGISTER_X0)
                });
            },
            Operation::Phi(vals) => {
                let ops = vals.iter().map(|v| self.get_vreg(*v, gen)).collect();
                gen.push_instr(Aarch64Instr::PhiPlaceholder { dst, ops });
            },
            _ => todo!(),
        }
    }

    fn select_terminator(
        &mut self,
        gen: &mut VCodeGenerator<Self::Instr>,
        term: &Terminator,
        func: &Function
    ) {
        match term {
            Terminator::Branch(val, t, f) => {
                let src1 = self.get_vreg(*val, gen);
                gen.push_instr(Aarch64Instr::Cbnz {
                    src1,
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
                let src = self.get_vreg(*val, gen);
                gen.push_instr(Aarch64Instr::MovReg {
                    dst: VReg::Real(AARCH64_REGISTER_X0),
                    src
                });
                gen.push_instr(Aarch64Instr::B { dst: LabelDest::Epilogue });
            }
            _ => todo!(),
        }
    }

    fn select_prologue(&mut self, gen: &mut VCodeGenerator<Self::Instr>, _func: &Function) {
        gen.push_instr(Aarch64Instr::AluImm {
            op: Aarch64AluOp::Sub,
            dst:  VReg::Real(AARCH64_REGISTER_SP),
            src1: VReg::Real(AARCH64_REGISTER_SP),
            src2: (AARCH64_CALLEE.len() * 16) as i64
        });

        for (i, r) in AARCH64_CALLEE.iter().enumerate() {
            gen.push_instr(Aarch64Instr::StoreSp {
                src: VReg::Real(*r),
                offset: (i * 16) as i64
            });
        }

        gen.push_instr(Aarch64Instr::MovReg {
            dst: VReg::Real(AARCH64_REGISTER_FP),
            src: VReg::Real(AARCH64_REGISTER_SP),
        });
    }

    fn select_epilogue(&mut self, gen: &mut VCodeGenerator<Self::Instr>, _func: &Function) {
        for (i, r) in AARCH64_CALLEE.iter().enumerate() {
            gen.push_instr(Aarch64Instr::LoadSp {
                dst: VReg::Real(*r),
                offset: (i * 16) as i64
            });
        }

        gen.push_instr(Aarch64Instr::AluImm {
            op: Aarch64AluOp::Add,
            dst:  VReg::Real(AARCH64_REGISTER_SP),
            src1: VReg::Real(AARCH64_REGISTER_SP),
            src2: (AARCH64_CALLEE.len() * 16) as i64
        });

        gen.push_instr(Aarch64Instr::Autibsp);
        gen.push_instr(Aarch64Instr::Ret);
    }
}

impl Aarch64Selector {
    pub fn get_vreg(&mut self, val: ValueId, gen: &mut VCodeGenerator<Aarch64Instr>) -> VReg {
        // VReg::Virtual(val.0)
        *self.virtual_map.entry(val.0).or_insert_with(|| gen.push_vreg())
    }
}

#[derive(Default)]
pub struct Aarch64Formatter;

impl DisplayVCode<Aarch64Instr> for Aarch64Formatter {
    fn fmt_inst(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        vcode: &VCode<Aarch64Instr>
    ) -> std::fmt::Result {
        for func in vcode.functions.iter() {
            match func.linkage {
                Linkage::External => {
                    writeln!(f, ".extern {}", func.name)?;
                    continue;
                },
                Linkage::Public => writeln!(f, ".global {}", func.name)?,
                Linkage::Private => {}
            }

            writeln!(f, "{}:", func.name)?;
            writeln!(f, "  .prologue:")?;
            for instr in func.prologue.instrs.iter() {
                write!(f, "    ")?;
                instr.fmt_inst(f, vcode)?;
                writeln!(f)?;
            }
            for (i, instrs) in func.instrs.iter().enumerate() {
                writeln!(f, "  .L{}:", i)?;
                for instr in instrs.instrs.iter() {
                    write!(f, "    ")?;
                    instr.fmt_inst(f, vcode)?;
                    writeln!(f)?;
                }
            }
            writeln!(f, "  .epilogue:")?;
            for instr in func.epilogue.instrs.iter() {
                write!(f, "    ")?;
                instr.fmt_inst(f, vcode)?;
                writeln!(f)?;
            }
        }
        Ok(())
    }
}
