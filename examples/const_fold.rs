use ssa::{
    algos::opt::*,
    builder::ModuleBuilder,
    ir::{BinOp, Terminator, Type},
};

fn main() {
    let mut builder = ModuleBuilder::new("fib");

    let (m_fn, _) = builder.push_function("main", Type::Void, vec![], None);
    builder.switch_to_fn(m_fn);

    const INT: Type = Type::Integer(16, false);

    let bb = builder.push_block();
    builder.switch_to_block(bb);

    let one = builder.build_integer(1, INT);
    let two = builder.build_integer(2, INT);
    let three = builder.build_integer(3, INT);

    let a = builder.build_binop(BinOp::Add, one, two, INT);
    let b = builder.build_binop(BinOp::Add, a, three, INT);

    builder.set_terminator(Terminator::Return(b));

    // builder.print_module();

    let mut module = builder.build();

    // module.apply_mandatory_transforms();
    ssa::algos::remove_critical_edges::remove_critical_edges(&mut module);
    ssa::algos::lower_to_ssa::lower(&mut module);

    println!("{module}");

    constant_folding::ConstantFolding.run(&mut module);
    println!("{module}");
}
