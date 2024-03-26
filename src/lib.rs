#![allow(dead_code)]

pub mod algos;
pub mod arch;
pub mod builder;
pub mod ir;
pub mod regalloc;
pub mod vcode;

#[cfg(test)]
mod tests {
    use crate::{
        builder::ModuleBuilder,
        ir::{Terminator, Type},
    };

    #[test]
    fn par_copy() {
        let mut builder = ModuleBuilder::new("par_copy");

        let (m_fn, _) = builder.push_function("main", Type::Void, vec![], None);
        builder.switch_to_fn(m_fn);

        let a = builder.push_block();
        let b = builder.push_block();

        builder.switch_to_block(a);
        let zero = builder.build_integer(0, Type::Integer(32, false));
        let one = builder.build_integer(1, Type::Integer(32, false));
        builder.set_terminator(Terminator::Jump(b));

        builder.switch_to_block(b);
        let x = builder.push_value(Type::Integer(32, false));
        let y = builder.push_value(Type::Integer(32, false));

        builder.module.functions[0].blocks[1]
            .instructions
            .push(crate::ir::Instruction {
                yielded: Some(x),
                operation: crate::ir::Operation::Phi(vec![one, y]),
            });
        builder.module.functions[0].blocks[1]
            .instructions
            .push(crate::ir::Instruction {
                yielded: Some(y),
                operation: crate::ir::Operation::Phi(vec![zero, x]),
            });
        builder.set_terminator(Terminator::Jump(b));

        builder.print_module();
        let mut module = builder.build();
        module.apply_mandatory_transforms();
        println!("{}", module);
    }

    #[test]
    fn test_var_renaming() {
        let mut builder = ModuleBuilder::new("test");
        let (f, _) = builder.push_function("main", Type::Void, vec![], None);
        builder.switch_to_fn(f);
        let entry = builder.push_block();
        builder.switch_to_block(entry);
        let x = builder.push_variable("x", Type::Integer(32, true));
        let three = builder.build_integer(3, Type::Integer(32, true));
        builder.build_store(x, three);
        let ld_x = builder.build_load(x);
        builder.set_terminator(Terminator::Return(ld_x));
        builder.print_module();
        let mut module = builder.build();
        module.apply_mandatory_transforms();
        println!("{}", module);
    }
}
