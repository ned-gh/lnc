use std::collections::HashMap;

use crate::parse::{Address, Instruction, ParseInfo};

pub fn assemble(parse_info: &ParseInfo) -> Result<[usize; 100], String> {
    if parse_info.instructions.len() >= 100 {
        return Err(format!(
            "Too many instructions: {} > 100",
            parse_info.instructions.len()
        ));
    }

    let mut mem = [0; 100];

    for (paddr, ins) in parse_info.instructions.iter().enumerate() {
        let code = match ins {
            Instruction::Load(addr) => 500 + resolve_addr(addr, &parse_info.label_map)?,
            Instruction::Store(addr) => 300 + resolve_addr(addr, &parse_info.label_map)?,
            Instruction::Add(addr) => 100 + resolve_addr(addr, &parse_info.label_map)?,
            Instruction::Subtract(addr) => 200 + resolve_addr(addr, &parse_info.label_map)?,
            Instruction::Input => 901,
            Instruction::Output => 902,
            Instruction::Halt => 0,
            Instruction::BranchZero(addr) => 700 + resolve_addr(addr, &parse_info.label_map)?,
            Instruction::BranchPositive(addr) => 800 + resolve_addr(addr, &parse_info.label_map)?,
            Instruction::BranchAlways(addr) => 600 + resolve_addr(addr, &parse_info.label_map)?,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lex, parse};

    fn single(source: &str) -> usize {
        let tokens = lex::tokenize(source).unwrap();
        let parse_info = parse::parse(&tokens).unwrap();
        let mem = assemble(&parse_info).unwrap();
        mem[0]
    }

    #[test]
    fn assemble_with_addr() {
        assert_eq!(single("lda 01"), 501);
        assert_eq!(single("sto 02"), 302);
        assert_eq!(single("add 03"), 103);
        assert_eq!(single("sub 04"), 204);
        assert_eq!(single("brz 99"), 799);
        assert_eq!(single("brp 98"), 898);
        assert_eq!(single("bra 97"), 697);
    }

    #[test]
    fn assemble_without_addr() {
        assert_eq!(single("inp"), 901);
        assert_eq!(single("out"), 902);
        assert_eq!(single("hlt"), 000);
    }

    #[test]
    fn assemble_data() {
        assert_eq!(single("dat 123"), 123);
    }
}
