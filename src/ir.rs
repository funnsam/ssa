use std::{
    collections::HashSet,
    fmt::{Debug, Display},
    ops::Deref,
};

use crate::{
    regalloc::Regalloc,
    vcode::{InstrSelector, VCode, VCodeGenerator, VCodeInstr},
};

/// `Module` is the struct containing all the functions and info about the
/// passes run on the SSA.
///
/// It is intended to be generated by the `ModuleBuilder` struct and then have
/// `.apply_mandatory_transforms()` called on it to lower to SSA form
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub(crate) functions: Vec<Function>,
    pub name: String,
    pub(crate) algos_run: Vec<Algo>,
}

/// Algo contains everything run on the module, and is useful for some sanity checks
/// and debugging
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Algo {
    CriticalEdgeSplitting,
    PhiLowering,
    PhiRemoval,
    LowerParMoves,
}

impl Module {
    /// Makes a new module with the given name and functions
    pub fn new(name: &str, functions: Vec<Function>) -> Module {
        Module {
            functions,
            name: name.to_string(),
            algos_run: vec![],
        }
    }

    /// Applies the mandatory transforms to the module and lowers it to SSA form
    pub fn apply_mandatory_transforms(&mut self) {
        crate::algos::remove_critical_edges::remove_critical_edges(self);
        crate::algos::lower_to_ssa::lower(self);

        crate::algos::phi_removal::remove_phis(self);
    }

    /// Lowers the module to vcode using the given instruction selector.
    /// The instruction selector may be defined outside of this crate and used,
    /// as long as you implement the `InstrSelector` trait for it and define
    /// registers avaliable for use.
    pub fn lower_to_vcode<
        I: VCodeInstr,
        S: InstrSelector<Instr = I> + Default,
        R: Regalloc + Default,
    >(
        &self,
    ) -> VCode<I> {
        let mut gen = VCodeGenerator::new();
        let mut selector = S::default();
        for func in self.functions.iter() {
            let args = (0..func.args.len()).map(|a| ValueId(a)).collect();
            let f = gen.push_function(&func.name, func.linkage, args);
            gen.switch_to_func(f);

            let init = gen.push_block();
            gen.switch_to_block(init);
            selector.get_pre_function_instructions(&mut gen);

            for bb in func.blocks.iter() {
                let b = gen.push_block();
                gen.switch_to_block(b);

                for instr in bb.instructions.iter() {
                    selector.select(&mut gen, instr);
                }
                selector.select_terminator(&mut gen, &bb.terminator);
            }

            selector.get_post_function_instructions(&mut gen);
        }
        let mut v = gen.build();
        let mut regalloc = R::default();
        for func in v.functions.iter_mut() {
            for block in &func.instrs {
                for instr in &block.instrs {
                    instr.collect_registers(&mut regalloc);
                    regalloc.next_instr();
                }
            }

            let allocs = regalloc.alloc_regs::<I>();

            for block in func.instrs.iter_mut() {
                for instr in block.instrs.iter_mut() {
                    instr.apply_allocs(&allocs);
                }
            }

            regalloc.reset();
        }

        v
    }
}

/// The struct containing info about functions as well as its body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub(crate) ret_type: Type,
    pub(crate) args: Vec<(String, Type)>,
    pub(crate) blocks: Vec<BasicBlock>,
    pub(crate) linkage: Linkage,
    pub(crate) variables: Vec<Variable>,
    pub(crate) id: usize,
    pub(crate) values: Vec<Value>,
}

impl Function {
    pub(crate) fn new(
        name: &str,
        ret_type: Type,
        args: Vec<(String, Type)>,
        linkage: Linkage,
        variables: Vec<Variable>,
        id: usize,
    ) -> (Self, Vec<ValueId>) {
        let mut values = Vec::new();
        let arg_len = args.len();
        for i in args.iter() {
            values.push(Value {
                ty: i.1.clone(),
                children: vec![],
                owner: BlockId(0),
            });
        }

        (
            Self {
                name: name.to_string(),
                ret_type,
                args,
                blocks: vec![],
                linkage,
                variables,
                id,
                values,
            },
            (0..arg_len).map(|a| ValueId(a)).collect(),
        )
    }

    pub(crate) fn push_block(&mut self, block: BasicBlock) {
        self.blocks.push(block);
    }

    pub(crate) fn push_value(&mut self, ty: Type) -> ValueId {
        let id = self.values.len();
        self.values.push(Value {
            ty,
            children: vec![],
            owner: BlockId(0),
        });
        ValueId(id)
    }

    pub(crate) fn replace_children_with(&mut self, original: ValueId, to_replace_to: ValueId) {
        for bb in self.blocks.iter_mut() {
            for instr in bb.instructions.iter_mut() {
                match &mut instr.operation {
                    Operation::BinOp(_, ref mut lhs, ref mut rhs) => {
                        if *lhs == original {
                            *lhs = to_replace_to;
                        }
                        if *rhs == original {
                            *rhs = to_replace_to;
                        }
                    }
                    Operation::Call(_, ref mut args) => {
                        for arg in args.iter_mut() {
                            if *arg == original {
                                *arg = to_replace_to;
                            }
                        }
                    }
                    Operation::StoreVar(.., ref mut val) => {
                        if *val == original {
                            *val = to_replace_to;
                        }
                    }
                    Operation::Phi(ref mut vals) => {
                        vals.iter_mut().for_each(|val| {
                            if *val == original {
                                *val = to_replace_to;
                            }
                        });
                    }
                    _ => (),
                }
            }
            match bb.terminator {
                Terminator::Return(ref mut val) => {
                    if *val == original {
                        *val = to_replace_to;
                    }
                }
                Terminator::Branch(ref mut val, ..) => {
                    if *val == original {
                        *val = to_replace_to;
                    }
                }
                _ => (),
            }
        }
        let mut c = self.values[original.0].children.clone();
        self.values[to_replace_to.0].children.append(&mut c);
        self.values[original.0].children.clear();
    }

    pub fn replace_instruction(&mut self, block: BlockId, instr: usize, new_instr: Instruction) {
        self.blocks[block.0].instructions[instr] = new_instr;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub(crate) name: String,
    pub(crate) ty: Type,
    pub(crate) bbs_assign_to: HashSet<BlockId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Value {
    pub(crate) ty: Type,
    pub(crate) children: Vec<ValueId>,
    pub(crate) owner: BlockId,
}

/// The type of a value/variable.
///
/// `Type::Integer(/* size */ usize, /* signed */ bool)`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Void,
    Integer(usize, bool),
    Pointer(Box<Type>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BasicBlock {
    pub(crate) instructions: Vec<Instruction>,
    pub(crate) terminator: Terminator,
    pub(crate) preds: Vec<BlockId>,
    pub(crate) id: usize,
    pub(crate) par_moves: Vec<(ValueId, ValueId)>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Terminator {
    Return(ValueId),
    Jump(BlockId),
    Branch(ValueId, BlockId, BlockId),
    NoTerm,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Linkage {
    Public,
    Private,
    External,
}

impl Display for Linkage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Linkage::Public => "public",
                Linkage::Private => "private",
                Linkage::External => "external",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Instruction {
    pub yielded: Option<ValueId>,
    pub operation: Operation,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Operation {
    Integer(i64),
    BinOp(BinOp, ValueId, ValueId),
    Call(FunctionId, Vec<ValueId>),
    LoadVar(VariableId),
    StoreVar(VariableId, ValueId),
    Phi(Vec<ValueId>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl BinOp {
    pub(crate) fn operate(&self, a: i64, b: i64) -> Option<i64> {
        match self {
            Self::Add => Some(a.wrapping_add(b)),
            Self::Sub => Some(a.wrapping_sub(b)),
            Self::Mul => Some(a.wrapping_mul(b)),
            Self::Div => a.checked_div(b),
            Self::Mod => a.checked_rem(b),
            Self::And => Some(a & b),
            Self::Or => Some(a | b),
            Self::Xor => Some(a ^ b),
            Self::Shl => Some(a << b),
            Self::Shr => Some(a >> b),
            Self::Eq => Some((a == b) as _),
            Self::Ne => Some((a != b) as _),
            Self::Lt => Some((a < b) as _),
            Self::Le => Some((a <= b) as _),
            Self::Gt => Some((a > b) as _),
            Self::Ge => Some((a >= b) as _),
        }
    }
}

impl Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Add => write!(f, "add"),
            BinOp::Sub => write!(f, "sub"),
            BinOp::Mul => write!(f, "mul"),
            BinOp::Div => write!(f, "div"),
            BinOp::Mod => write!(f, "mod"),
            BinOp::And => write!(f, "and"),
            BinOp::Or => write!(f, "or"),
            BinOp::Xor => write!(f, "xor"),
            BinOp::Shl => write!(f, "shl"),
            BinOp::Shr => write!(f, "shr"),
            BinOp::Eq => write!(f, "eq"),
            BinOp::Ne => write!(f, "ne"),
            BinOp::Lt => write!(f, "lt"),
            BinOp::Le => write!(f, "le"),
            BinOp::Gt => write!(f, "gt"),
            BinOp::Ge => write!(f, "ge"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub(crate) usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(pub(crate) usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VariableId(pub(crate) usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueId(pub(crate) usize);

impl Deref for BlockId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for FunctionId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for VariableId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for ValueId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "/* {:?} module {} */", self.algos_run, self.name)?;

        for func in &self.functions {
            writeln!(f, "{}", func)?;
        }

        Ok(())
    }
}

impl Debug for Algo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Algo::CriticalEdgeSplitting => {
                write!(f, "@edges_splitted")
            }
            Algo::PhiLowering => {
                write!(f, "@phis_lowered")
            }
            Algo::PhiRemoval => {
                write!(f, "@phis_removed")
            }
            Algo::LowerParMoves => {
                write!(f, "@par_moves_lowered")
            }
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "${}: {} fn {}({}) {} {{",
            self.id,
            self.linkage,
            self.name,
            self.args
                .iter()
                .map(|e| format!("{}: {}", e.0, e.1))
                .collect::<Vec<String>>()
                .join(", "),
            self.ret_type
        )?;

        for block in &self.blocks {
            write!(f, "{}", block)?;
        }

        write!(f, "}}")?;
        Ok(())
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Void => write!(f, "void")?,
            Type::Integer(size, signed) => {
                write!(f, "{}{}", if *signed { "s" } else { "u" }, size)?
            }
            Type::Pointer(ty) => write!(f, "{}*", ty)?,
        }
        Ok(())
    }
}

impl Display for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "${}: ; preds = {}",
            self.id,
            self.preds
                .iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<String>>()
                .join(", ")
        )?;
        for instr in &self.instructions {
            writeln!(f, "    {}", instr)?;
        }
        if !self.par_moves.is_empty() {
            let tmp = self
                .par_moves
                .iter()
                .fold((Vec::new(), Vec::new()), |acc, elem| {
                    let mut acc = acc;
                    acc.0.push(format!("{}", elem.0));
                    acc.1.push(format!("{}", elem.1));
                    acc
                });
            writeln!(f, "    {:?} <- {:?}", tmp.0, tmp.1)?;
        }
        writeln!(f, "    {}", self.terminator)?;
        Ok(())
    }
}

impl Display for Terminator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Terminator::Return(var) => write!(f, "ret {}", var)?,
            Terminator::Jump(block) => write!(f, "jmp ${}", block.0)?,
            Terminator::Branch(var, block1, block2) => {
                write!(f, "br {}, ${}, ${}", var, block1.0, block2.0)?
            }
            Terminator::NoTerm => write!(f, "noterm")?,
        }
        Ok(())
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(val) = self.yielded {
            write!(f, "{} = ", val)?;
        }
        write!(f, "{}", self.operation)
    }
}

impl Display for ValueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%{}", self.0)
    }
}

impl Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.0)
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::BinOp(op, lhs, rhs) => write!(f, "{} {} {}", op, lhs, rhs)?,
            Operation::Call(func, args) => write!(
                f,
                "call ${}({})",
                func.0,
                args.iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<String>>()
                    .join(", ")
            )?,
            Operation::LoadVar(var) => write!(f, "load #{}", var.0)?,
            Operation::StoreVar(var, val) => write!(f, "store #{} {}", var.0, val)?,
            Operation::Integer(val) => write!(f, "{}", val)?,
            Operation::Phi(vals) => write!(
                f,
                "Φ {}",
                vals.iter()
                    .map(|val| format!("{}", val))
                    .collect::<Vec<String>>()
                    .join(", ")
            )?,
        }
        Ok(())
    }
}
