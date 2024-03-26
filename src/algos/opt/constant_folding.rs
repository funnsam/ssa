use std::collections::HashMap;

use super::OptPass;
use crate::ir::*;

pub struct ConstantFolding;

impl OptPass for ConstantFolding {
    fn run(&mut self, module: &mut Module) {
        for f in module.functions.iter_mut() {
            let mut known_values = HashMap::new();

            for b in f.blocks.iter_mut() {
                for i in b.instructions.iter_mut() {
                    match i.operation {
                        Operation::Integer(int) => {
                            known_values.insert(i.yielded.unwrap(), int);
                        }
                        Operation::BinOp(op, a, b) => {
                            if let (Some(av), Some(bv)) =
                                (known_values.get(&a), known_values.get(&b))
                            {
                                if let Some(result) = op.operate(*av, *bv) {
                                    known_values.insert(i.yielded.unwrap(), result);
                                    *i = Instruction {
                                        operation: Operation::Integer(result),
                                        yielded: i.yielded,
                                    };
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
