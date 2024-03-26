use ssa::{
    arch::iris::IrisSelector,
    builder::ModuleBuilder,
    ir::{BinOp, Terminator, Type},
    regalloc::linear_scan::LinearScanRegAlloc,
};

fn main() {
    const INT: Type = Type::Integer(16, false);

    let mut builder = ModuleBuilder::new("fib");

    let (fib, args) = builder.push_function("fib", Type::Void, vec![("nth".to_string(), INT)], None);
    builder.switch_to_fn(fib);

    // LIGHT:
    // fib(%nth):
    //   Init:
    //     %one = 1
    //     store #x %one
    //     %zero = 0
    //     store #y %zero
    //     store #cnt %nth
    //   Loop:
    //     %x = load #x
    //     %y = load #y
    //     %nx = %x + %y
    //     store #x %nx
    //     %ny = %nx - %y
    //     store #y %ny
    //     %cnt = load #cnt
    //     %ncnt = %cnt - 1
    //     store #cnt %ncnt
    //     branch %ncnt $Loop $End
    //   End:
    //     return

    let init_bb = builder.push_block();
    let loop_bb = builder.push_block();
    let end_bb = builder.push_block();

    let x = builder.push_variable("x", INT);
    let y = builder.push_variable("y", INT);
    let cnt = builder.push_variable("cnt", INT);

    builder.switch_to_block(init_bb);
    let one = builder.build_integer(1, INT);
    builder.build_store(x, one);
    builder.build_store(y, one);
    builder.build_store(cnt, args[0]);
    builder.set_terminator(Terminator::Jump(loop_bb));

    builder.switch_to_block(loop_bb);
    let xv = builder.build_load(x);
    let yv = builder.build_load(y);
    let nx = builder.build_binop(BinOp::Add, xv, yv, INT);
    builder.build_store(x, nx);
    let ny = builder.build_binop(BinOp::Sub, nx, yv, INT);
    builder.build_store(y, ny);
    let c = builder.build_load(cnt);
    let nc = builder.build_binop(BinOp::Sub, c, one, INT);
    builder.build_store(cnt, nc);
    builder.set_terminator(Terminator::Branch(nc, loop_bb, end_bb));

    builder.switch_to_block(end_bb);
    builder.set_terminator(Terminator::Return(nx));

    let (m_fn, _) = builder.push_function("main", INT, vec![], Some(ssa::ir::Linkage::Public));
    builder.switch_to_fn(m_fn);
    let bb = builder.push_block();
    builder.switch_to_block(bb);
    let ret = builder.build_call(fib, vec![]);
    builder.set_terminator(Terminator::Return(ret));

    builder.print_module();

    let mut module = builder.build();
    module.apply_mandatory_transforms();
    println!("{module}");

    let vcode = module.lower_to_vcode::<_, IrisSelector, LinearScanRegAlloc>();
    println!("{}", vcode);
}
