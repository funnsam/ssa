use ssa::{
    arch::iris::IrisSelector,
    builder::ModuleBuilder,
    ir::{BinOp, Terminator, Type},
    regalloc::linear_scan::LinearScanRegAlloc,
};

fn main() {
    let mut builder = ModuleBuilder::new("fib");

    let m_fn = builder.push_function("main", Type::Void, vec![], None);
    builder.switch_to_fn(m_fn);

    // LIGHT:
    // Init:
    //     %one = 1
    //     store #x %one
    //     %zero = 0
    //     store #y %zero
    //     %nth = 10
    //     store #cnt %nth
    // Loop:
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
    // End:
    //     return

    const INT: Type = Type::Integer(16, false);

    let init_bb = builder.push_block();
    let loop_bb = builder.push_block();
    let end_bb = builder.push_block();

    let x = builder.push_variable("x", INT);
    let y = builder.push_variable("y", INT);
    let cnt = builder.push_variable("cnt", INT);

    builder.switch_to_block(init_bb);
    let one = builder.build_integer(1, INT);
    builder.build_store(x, one);
    let zero = builder.build_integer(0, INT);
    builder.build_store(y, zero);
    let iter = builder.build_integer(10, INT);
    builder.build_store(cnt, iter);
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
    builder.set_terminator(Terminator::Return(zero));

    builder.print_module();

    let vcode = builder.build().lower_to_vcode::<_, IrisSelector, LinearScanRegAlloc>();
    println!("{}", vcode);
}
