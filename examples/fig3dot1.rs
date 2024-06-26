use ssa::{
    arch::iris::IrisSelector,
    builder::ModuleBuilder,
    ir::{BinOp, Terminator, Type},
    regalloc::linear_scan::LinearScanRegAlloc,
};

fn main() {
    let mut builder = ModuleBuilder::new("fig3.1");

    let (m_fn, _) = builder.push_function("main", Type::Void, vec![], None);
    builder.switch_to_fn(m_fn);

    let entry = builder.push_block();
    let bb_a = builder.push_block();
    let bb_b = builder.push_block();
    let bb_c = builder.push_block();
    let bb_d = builder.push_block();
    let bb_e = builder.push_block();

    builder.switch_to_block(entry);
    builder.set_terminator(Terminator::Jump(bb_a));

    builder.switch_to_block(bb_b);
    let x = builder.push_variable("x", Type::Integer(32, true)); // i32
    let y = builder.push_variable("y", Type::Integer(32, true)); // i32
    let val = builder.build_integer(0, Type::Integer(4, true));
    builder.build_store(x, val);
    builder.build_store(y, val);
    builder.set_terminator(Terminator::Jump(bb_d));

    builder.switch_to_block(bb_c);
    let tmp = builder.push_variable("tmp", Type::Integer(32, true));
    let ld_x = builder.build_load(x);
    let ld_y = builder.build_load(y);
    builder.build_store(tmp, ld_x);
    builder.build_store(x, ld_y);
    let ld_tmp = builder.build_load(tmp);
    builder.build_store(y, ld_tmp);
    let ld_x = builder.build_load(x);
    builder.set_terminator(Terminator::Branch(ld_x, bb_d, bb_e));

    builder.switch_to_block(bb_d);
    let ld_x = builder.build_load(x);
    let ld_y = builder.build_load(y);
    let val = builder.build_binop(BinOp::Add, ld_x, ld_y, Type::Integer(4, true));
    builder.build_store(x, val);
    let ld_x = builder.build_load(x);
    builder.set_terminator(Terminator::Branch(ld_x, bb_a, bb_e));

    builder.switch_to_block(bb_e);
    let ld_x = builder.build_load(x);
    builder.set_terminator(Terminator::Return(ld_x));

    builder.switch_to_block(bb_a);
    let ld_tmp = builder.build_load(tmp);
    builder.set_terminator(Terminator::Branch(ld_tmp, bb_b, bb_c));

    builder.print_module();

    let mut module = builder.build();
    module.apply_mandatory_transforms();
    println!("{}", module);
    let vcode = module.lower_to_vcode::<_, IrisSelector, LinearScanRegAlloc>();
    println!("{}", vcode);
}
