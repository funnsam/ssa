use ssa::{
    algos::lower_to_ssa,
    arch::aarch64::Aarch64Selector,
    builder::ModuleBuilder,
    ir::{BinOp, Terminator, Type, Linkage}, regalloc::linear_scan::LinearScanRegAlloc,
    vcode::DisplayVCode,
};

fn main() {
    let mut builder = ModuleBuilder::new("test");
    let putchar = builder.push_function(
        "putchar",
        Type::Void,
        vec![("char".to_string(), Type::Integer(32, true))],
        Some(Linkage::External)
    );
    let main = builder.push_function("main", Type::Integer(32, true), vec![], Some(Linkage::Public));
    builder.switch_to_fn(main);

    let entry = builder.push_block("entry");
    builder.switch_to_block(entry);

    let a_char = builder.build_integer(b'a' as i64, Type::Integer(32, true));
    let lf_char = builder.build_integer(b'\n' as i64, Type::Integer(32, true));
    builder.build_call(putchar, vec![a_char]);
    builder.build_call(putchar, vec![lf_char]);

    let exit = builder.push_block("exit");
    builder.set_terminator(Terminator::Jump(exit));
    builder.switch_to_block(exit);

    let x = builder.push_variable("x", Type::Integer(32, true));
    let y = builder.push_variable("y", Type::Integer(32, true));
    let three = builder.build_integer(3, Type::Integer(32, true));
    builder.build_store(x, three);
    builder.build_store(y, three);
    let ld_x = builder.build_load(x);
    let ld_y = builder.build_load(y);
    let val = builder.build_binop(BinOp::Add, ld_x, ld_y, Type::Integer(32, true));

    builder.set_terminator(Terminator::Return(val));
    let mut module = builder.build();
    module.apply_mandatory_transforms();
    eprintln!("{}", module);
    let vcode = module.lower_to_vcode::<_, Aarch64Selector, LinearScanRegAlloc>();
    println!("{}", vcode.to_fmt(&vcode));
}
