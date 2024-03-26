use crate::{
    algos::delete_instructions::delete,
    ir::{Algo, Module, Operation},
};

pub fn remove_phis(module: &mut Module) {
    // Modules without critical edge splitting will have program semantics changed if phis are removed
    assert!(module.algos_run.contains(&Algo::CriticalEdgeSplitting));
    module.algos_run.push(Algo::PhiRemoval);
    // Function: V: BB: I -> should delete instruction
    let mut func_dels: Vec<Vec<Vec<bool>>> = Vec::new();

    for (func_id, func) in module.functions.iter_mut().enumerate() {
        func_dels.push(Vec::new());
        for (block_id, block) in func.blocks.clone().into_iter().enumerate() {
            func_dels[func_id].push(Vec::new());
            for (instr_id, instr) in block.instructions.into_iter().enumerate() {
                func_dels[func_id][block_id].push(false);
                match &instr.operation {
                    Operation::Phi(defs) => {
                        for val in defs {
                            let target_block = func.values[val.0].owner;
                            func.blocks[target_block.0]
                                .par_moves
                                .push((instr.yielded.unwrap(), val.clone()))
                        }
                        func_dels[func_id][block_id][instr_id] = true;
                    }
                    _ => continue,
                }
            }
        }
    }

    delete(module, &func_dels);

    for (_, func) in module.functions.iter_mut().enumerate() {
        for (bi, block) in func.blocks.iter_mut().enumerate() {
            for m in super::par_move::parallel_move(&mut block.par_moves, &mut |a, _| {
                func.values.push(crate::ir::Value {
                    ty: func.values[a.0].ty.clone(),
                    children: vec![],
                    owner: crate::ir::BlockId(bi),
                });

                crate::ir::ValueId(func.values.len() - 1)
            }) {
                block.instructions.push(crate::ir::Instruction {
                    yielded: Some(m.0),
                    operation: Operation::BinOp(crate::ir::BinOp::And, m.1, m.1),
                });
            }
        }
    }
}
