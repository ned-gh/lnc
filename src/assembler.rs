use std::collections::HashMap;

use crate::parse::{Address, Instruction, ParseInfo};

fn assemble(parse_info: &ParseInfo) -> Result<[usize; 100], String> {
    if parse_info.instructions.len() >= 100 {
        return Err(format!(
            "Too many instructions: {} > 100",
            parse_info.instructions.len()
        ));
    }

    let mut mem = [0; 100];

    for (paddr, ins) in parse_info.instructions.iter().enumerate() {
        let code = match ins {
            Instruction::Load(addr) => 5 + resolve_addr(addr, &parse_info.label_map)?,
            Instruction::Store(addr) => 3 + resolve_addr(addr, &parse_info.label_map)?,
            Instruction::Add(addr) => 1 + resolve_addr(addr, &parse_info.label_map)?,
            Instruction::Subtract(addr) => 2 + resolve_addr(addr, &parse_info.label_map)?,
            Instruction::Input => 901,
            Instruction::Output => 902,
            Instruction::Halt => 0,
            Instruction::BranchZero(addr) => 7 + resolve_addr(addr, &parse_info.label_map)?,
            Instruction::BranchPositive(addr) => 8 + resolve_addr(addr, &parse_info.label_map)?,
            Instruction::BranchAlways(addr) => 6 + resolve_addr(addr, &parse_info.label_map)?,
            Instruction::Data(data) => *data,
        };

        mem[paddr] = code;
    }

    Ok(mem)
}

fn resolve_addr(addr: &Address, label_map: &HashMap<String, usize>) -> Result<usize, String> {
    match addr {
        Address::Symbolic(label) => resolve_symb_addr(label, label_map),
        Address::Numeric(n) => Ok(*n),
    }
}

fn resolve_symb_addr(label: &str, label_map: &HashMap<String, usize>) -> Result<usize, String> {
    if let Some(addr) = label_map.get(label) {
        Ok(*addr)
    } else {
        Err(format!("Label '{}' is not defined", label))
    }
}
