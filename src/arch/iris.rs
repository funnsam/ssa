//! # Iris default calling convention:
//! - `R1`: return value
//! - `R1` - `R8`: arguments / caller save
//! - `R9` - `R25`: scratch register / caller save

use std::fmt::Display;

use crate::{
    algos::par_move::parallel_move,
    ir::*,
    regalloc::{apply_alloc, VReg},
    vcode::*,
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

pub const IRIS_REG_ARGS: &[usize] = &[
    IRIS_REG_1, IRIS_REG_2, IRIS_REG_3, IRIS_REG_4, IRIS_REG_5, IRIS_REG_6, IRIS_REG_7, IRIS_REG_8,
];

pub const IRIS_REGS: &[VReg] = &[
    VReg::Real(IRIS_REG_1),
    VReg::Real(IRIS_REG_2),
    VReg::Real(IRIS_REG_3),
    VReg::Real(IRIS_REG_4),
    VReg::Real(IRIS_REG_5),
    VReg::Real(IRIS_REG_6),
    // VReg::Real(IRIS_REG_7),
    // VReg::Real(IRIS_REG_8),
    // VReg::Real(IRIS_REG_9),
    // VReg::Real(IRIS_REG_10),
    // VReg::Real(IRIS_REG_11),
    // VReg::Real(IRIS_REG_12),
    // VReg::Real(IRIS_REG_13),
    // VReg::Real(IRIS_REG_14),
    // VReg::Real(IRIS_REG_15),
    // VReg::Real(IRIS_REG_16),
    // VReg::Real(IRIS_REG_17),
    // VReg::Real(IRIS_REG_18),
    // VReg::Real(IRIS_REG_19),
    // VReg::Real(IRIS_REG_20),
    // VReg::Real(IRIS_REG_21),
    // VReg::Real(IRIS_REG_22),
    // VReg::Real(IRIS_REG_23),
    // VReg::Real(IRIS_REG_24),
    // VReg::Real(IRIS_REG_25),
    // VReg::Real(IRIS_REG_26),
];

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
        cond: VReg,
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
    HPsh {
        val: VReg,
    },
    HPop {
        dst: VReg,
    },
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
        IRIS_REGS
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
            Self::Beq { cond, .. } => {
                regalloc.add_use(*cond);
            }
            Self::Imm { dst, .. } => {
                regalloc.add_def(*dst);
            }
            Self::Mov { dst, src } => {
                regalloc.add_def(*dst);
                regalloc.add_use(*src);
                regalloc.coalesce_move(*src, *dst);
            }
            Self::HPsh { val } => {
                regalloc.add_use(*val);
            }
            Self::HPop { dst } => {
                regalloc.add_def(*dst);
            }
            Self::Jmp { .. } | Self::PhiPlaceholder { .. } | Self::Ret | Self::Cal { .. } => {},
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
            Self::Beq { cond, .. } => {
                apply_alloc(cond, allocs);
            }
            Self::Imm { dst, .. } => {
                apply_alloc(dst, allocs);
            }
            Self::Mov { dst, src } => {
                apply_alloc(dst, allocs);
                apply_alloc(src, allocs);
            }
            Self::HPsh { val } => {
                apply_alloc(val, allocs);
            }
            Self::HPop { dst } => {
                apply_alloc(dst, allocs);
            }
            Self::Jmp { .. } | Self::PhiPlaceholder { .. } | Self::Ret | Self::Cal { .. } => {},
        }
    }

    fn apply_mandatory_transforms(vcode: &mut VCode<Self>) {
        // TODO: spilled stuff
    }

    fn emit_assembly<T: std::io::Write>(w: &mut T, vcode: &VCode<Self>) -> std::io::Result<()> {
        fn mangle<I: VCodeInstr>(vcode: &VCode<I>, f: &VCodeFunction<I>, l: &LabelDest) -> String {
            fn mangle_string(s: &str) -> String {
                use std::hash::*;
                let mut h = DefaultHasher::new();
                s.hash(&mut h);
                format!("{s}_{:16x}", h.finish())
            }

            match l {
                LabelDest::Block(li) => mangle_string(&format!(".__fn_{}{}_L{}", f.name, f.arg_count, li.0)),
                LabelDest::Function(fi) => match vcode.functions[fi.0].linkage {
                    Linkage::Private => format!(".{}", mangle_string(&vcode.functions[fi.0].name)),
                    _ => format!(".{}", vcode.functions[fi.0].name.clone()),
                },
            }
        }

        writeln!(w, "cal .main")?;
        writeln!(w, "hlt")?;

        for (fi, f) in vcode.functions.iter().enumerate() {
            if matches!(f.linkage, Linkage::External) {
                continue;
            }

            writeln!(w, "{}", mangle(vcode, &f, &LabelDest::Function(FunctionId(fi))))?;
            for (li, l) in f.instrs.iter().enumerate() {
                if li != 0 {
                    writeln!(w, "{}", mangle(vcode, &f, &LabelDest::Block(BlockId(li - 1))))?;
                }

                for i in l.instrs.iter() {
                    match i {
                        IrisInstr::Jmp { dst } => writeln!(w, "jmp {}", mangle(vcode, &f, dst))?,
                        IrisInstr::Beq { cond: src1, dst } => writeln!(w, "bnz {} {}", mangle(vcode, &f, dst), src1)?,
                        IrisInstr::Cal { dst } => writeln!(w, "cal {}", mangle(vcode, &f, dst))?,
                        _ => writeln!(w, "{i}")?,
                    }
                }
            }
        }

        Ok(())
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
                write!(f, "{op} {dst} {src1} {src2}")
            }
            IrisInstr::Jmp { dst } => write!(f, "jmp {dst}"),
            IrisInstr::Imm { dst, val } => write!(f, "imm {dst} {val}"),
            IrisInstr::Beq { cond, dst } => write!(f, "bnz {dst} {cond}"),
            IrisInstr::Mov { dst, src } => write!(f, "mov {dst} {src}"),
            IrisInstr::Cal { dst } => write!(f, "cal {dst}"),
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
            IrisInstr::HPsh { val } => write!(f, "hpsh {val}"),
            IrisInstr::HPop { dst } => write!(f, "hpop {dst}"),
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
        let dst = instr
            .yielded
            .map_or(VReg::Real(IRIS_REG_ZR), |val| self.get_vreg(val));

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
            Operation::LoadVar(..) | Operation::StoreVar(..) => unreachable!(),
            // THESE NEVER GET EXECUTED (removed in algos::lower_to_ssa::lower())
            Operation::Phi(vals) => {
                gen.push_instr(IrisInstr::PhiPlaceholder {
                    dst,
                    ops: vals.iter().map(|v| self.get_vreg(*v)).collect(),
                });
            }
            Operation::Call(f, args) => {
                // TODO: save regs
                for r in IRIS_REGS.iter() {
                    gen.push_instr(IrisInstr::HPsh { val: *r });
                }

                for (dst, src) in parallel_move(
                    &mut IRIS_REG_ARGS
                        .iter()
                        .map(|a| VReg::Real(*a))
                        .zip(args.iter().map(|a| self.get_vreg(*a)))
                        .collect(),
                    &mut |_, _| gen.push_vreg(),
                ) {
                    gen.push_instr(IrisInstr::Mov { dst, src });
                }

                gen.push_instr(IrisInstr::Cal {
                    dst: LabelDest::Function(*f),
                });

                // TODO: preserve return value
                gen.push_instr(IrisInstr::Mov {
                    dst: self.get_vreg(instr.yielded.unwrap()),
                    src: VReg::Real(IRIS_REG_1)
                });

                for r in IRIS_REGS.iter().rev() {
                    gen.push_instr(IrisInstr::HPop { dst: *r });
                }
            }
        }
    }

    fn select_terminator(&mut self, gen: &mut VCodeGenerator<Self::Instr>, term: &Terminator) {
        match term {
            Terminator::Branch(val, t, f) => {
                gen.push_instr(IrisInstr::Beq {
                    cond: self.get_vreg(*val),
                    dst: LabelDest::Block(*t),
                });
                gen.push_instr(IrisInstr::Jmp {
                    dst: LabelDest::Block(*f),
                });
            }
            Terminator::Jump(l) => {
                gen.push_instr(IrisInstr::Jmp {
                    dst: LabelDest::Block(*l),
                });
            }
            Terminator::Return(val) => {
                gen.push_instr(IrisInstr::Mov {
                    dst: VReg::Real(IRIS_REG_1),
                    src: self.get_vreg(*val),
                });
                gen.push_instr(IrisInstr::Ret);
            }
            Terminator::NoTerm => {}
        }
    }

    fn get_pre_function_instructions(&mut self, gen: &mut VCodeGenerator<Self::Instr>) {
        for (dst, src) in parallel_move(
            &mut gen
            .args
            .iter()
            .map(|a| self.get_vreg(*a))
            .zip(IRIS_REG_ARGS.iter().map(|a| VReg::Real(*a)))
            .collect(),
            &mut |_, _| gen.push_vreg(),
        ) {
            gen.push_instr(IrisInstr::Mov { dst, src });
        }
    }

    fn get_post_function_instructions(&mut self, gen: &mut VCodeGenerator<Self::Instr>) {}
}

impl IrisSelector {
    #[inline]
    pub fn get_vreg(&self, val: ValueId) -> VReg {
        VReg::Virtual(val.0)
    }
}
