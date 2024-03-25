use crate::ir::Module;

pub fn delete(module: &mut Module, dels: &[Vec<Vec<bool>>]) {
    for (fi, func) in dels.iter().enumerate() {
        for (bi, block) in func.iter().enumerate() {
            let mut deleted = 0;
            for (i, inst) in block.iter().enumerate() {
                if *inst {
                    module.functions[fi].blocks[bi].instructions.remove(i - deleted);
                    deleted += 1;
                }
            }
        }
    }
}
