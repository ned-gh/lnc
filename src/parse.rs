use std::collections::HashMap;
use std::iter::Peekable;
use std::slice::Iter;

use crate::lex::Token;

#[derive(Debug, PartialEq, Eq)]
enum Address {
    Symbolic(String),
    Numeric(usize),
}

#[derive(Debug, PartialEq, Eq)]
enum InstructionKind {
    Load(Address),
    Store(Address),
    Add(Address),
    Subtract(Address),
    Input,
    Output,
    Halt,
    BranchZero(Address),
    BranchPositive(Address),
    BranchAlways(Address),
    Data(usize),
}

#[derive(Debug, PartialEq, Eq)]
struct Instruction {
    kind: InstructionKind,
    addr: usize,
}

struct ParseInfo {
    instructions: Vec<Instruction>,
    label_map: HashMap<String, usize>,
}

impl ParseInfo {
    fn new() -> Self {
        Self {
            instructions: vec![],
            label_map: HashMap::new(),
        }
    }
}

struct Parser<'a> {
    it: Peekable<Iter<'a, Token>>,
    paddr: usize,
    info: ParseInfo,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self {
            it: tokens.iter().peekable(),
            paddr: 0,
            info: ParseInfo::new(),
        }
    }

    fn make_instructions(mut self) -> Result<ParseInfo, String> {
        while let Some(token) = self.consume() {
            match token {
                Token::LabelDef(s) => {
                    self.info.label_map.insert(s, self.paddr);
                }
                Token::Load
                | Token::Store
                | Token::Add
                | Token::Subtract
                | Token::BranchZero
                | Token::BranchPositive
                | Token::BranchAlways => self.ins_with_addr(token)?,
                Token::Input | Token::Output | Token::Halt => self.ins_without_addr(token)?,
                Token::Data => self.data()?,
                Token::NewLine => (),
                Token::Eof => break,
                _ => return Err(format!("Invalid token {:?}", token)),
            }
        }

        Ok(self.info)
    }

    fn consume(&mut self) -> Option<Token> {
        self.it.next().cloned()
    }

    fn peek(&mut self) -> Option<&Token> {
        self.it.peek().copied()
    }

    fn add_ins(&mut self, kind: InstructionKind) {
        self.info.instructions.push(Instruction {
            kind,
            addr: self.paddr,
        });
        self.paddr += 1;
    }

    fn check_newline(&mut self) -> Result<(), String> {
        if let Some(nl_token) = self.consume() {
            if !matches!(nl_token, Token::NewLine | Token::Eof) {
                return Err(format!("Encountered {:?} instead of new line", nl_token));
            }
        } else {
            return Err("No token found".to_owned());
        }

        Ok(())
    }

    fn ins_with_addr(&mut self, token: Token) -> Result<(), String> {
        let addr = if let Some(addr_token) = self.consume() {
            match addr_token {
                Token::Number(n) => {
                    if n >= 100 {
                        return Err(format!("Invalid address {}: too large", n));
                    }
                    Address::Numeric(n)
                }
                Token::Label(s) => Address::Symbolic(s),
                _ => return Err(format!("Invalid token {:?}: expected address", addr_token)),
            }
        } else {
            return Err("No token found".to_owned());
        };

        self.check_newline()?;

        match token {
            Token::Load => self.add_ins(InstructionKind::Load(addr)),
            Token::Store => self.add_ins(InstructionKind::Store(addr)),
            Token::Add => self.add_ins(InstructionKind::Add(addr)),
            Token::Subtract => self.add_ins(InstructionKind::Subtract(addr)),
            Token::BranchZero => self.add_ins(InstructionKind::BranchZero(addr)),
            Token::BranchPositive => self.add_ins(InstructionKind::BranchPositive(addr)),
            Token::BranchAlways => self.add_ins(InstructionKind::BranchAlways(addr)),
            _ => unreachable!(),
        }

        Ok(())
    }

    fn ins_without_addr(&mut self, token: Token) -> Result<(), String> {
        self.check_newline()?;

        match token {
            Token::Input => self.add_ins(InstructionKind::Input),
            Token::Output => self.add_ins(InstructionKind::Output),
            Token::Halt => self.add_ins(InstructionKind::Halt),
            _ => unreachable!(),
        }

        Ok(())
    }

    fn data(&mut self) -> Result<(), String> {
        let num = if let Some(num_token) = self.consume() {
            if let Token::Number(n) = num_token {
                if n >= 1000 {
                    return Err(format!("Invalid data {}: too large", n));
                }
                n
            } else {
                return Err(format!("Invalid token {:?}: expected number", num_token));
            }
        } else {
            return Err("No token found".to_owned());
        };

        self.add_ins(InstructionKind::Data(num));

        Ok(())
    }
}

fn parse(tokens: &[Token]) -> Result<ParseInfo, String> {
    let parser = Parser::new(tokens);
    parser.make_instructions()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::tokenize;

    fn single(source: &str) -> Instruction {
        parse_src(source).unwrap().instructions.remove(0)
    }

    fn parse_src(source: &str) -> Result<ParseInfo, String> {
        let tokens = tokenize(source).unwrap();
        parse(&tokens)
    }

    fn ins_at_zero(kind: InstructionKind) -> Instruction {
        Instruction { kind, addr: 0 }
    }

    #[test]
    fn parse_with_addr() {
        use Address::Numeric;
        use InstructionKind::*;

        assert_eq!(single("lda 01"), ins_at_zero(Load(Numeric(1))));
        assert_eq!(single("sto 02"), ins_at_zero(Store(Numeric(2))));
        assert_eq!(single("add 03"), ins_at_zero(Add(Numeric(3))));
        assert_eq!(single("sub 04"), ins_at_zero(Subtract(Numeric(4))));
        assert_eq!(single("brz 05"), ins_at_zero(BranchZero(Numeric(5))));
        assert_eq!(single("brp 06"), ins_at_zero(BranchPositive(Numeric(6))));
        assert_eq!(single("bra 07"), ins_at_zero(BranchAlways(Numeric(7))));
    }

    #[test]
    fn parse_with_symbolic_addr() {
        use Address::Symbolic;
        use InstructionKind::*;

        assert_eq!(
            single("lda this"),
            ins_at_zero(Load(Symbolic("this".into())))
        );
        assert_eq!(single("sto is"), ins_at_zero(Store(Symbolic("is".into()))));
        assert_eq!(single("add a"), ins_at_zero(Add(Symbolic("a".into()))));
        assert_eq!(
            single("sub test"),
            ins_at_zero(Subtract(Symbolic("test".into())))
        );
        assert_eq!(
            single("brz with"),
            ins_at_zero(BranchZero(Symbolic("with".into())))
        );
        assert_eq!(
            single("brp symbolic"),
            ins_at_zero(BranchPositive(Symbolic("symbolic".into())))
        );
        assert_eq!(
            single("bra addresses"),
            ins_at_zero(BranchAlways(Symbolic("addresses".into())))
        );
    }

    #[test]
    fn parse_without_addr() {
        use InstructionKind::*;

        assert_eq!(single("inp"), ins_at_zero(Input));
        assert_eq!(single("out"), ins_at_zero(Output));
        assert_eq!(single("hlt"), ins_at_zero(Halt));
    }

    #[test]
    fn parse_data() {
        use InstructionKind::Data;

        assert_eq!(single("dat 123"), ins_at_zero(Data(123)));
    }

    #[test]
    fn parse_handles_newlines() {
        use Address::Numeric;
        use InstructionKind::*;

        let source = "
            lda 10 ; this is
            add 11 ; a comment

            sto 10
            hlt";
        let info = parse_src(source).unwrap();
        let expected = vec![
            Instruction {
                kind: Load(Numeric(10)),
                addr: 0,
            },
            Instruction {
                kind: Add(Numeric(11)),
                addr: 1,
            },
            Instruction {
                kind: Store(Numeric(10)),
                addr: 2,
            },
            Instruction {
                kind: Halt,
                addr: 3,
            },
        ];

        assert_eq!(info.instructions, expected);
    }

    #[test]
    fn fails_on_bad_ops() {
        // kw as addr
        assert!(parse_src("lda add").is_err());

        // labeldef as addr
        assert!(parse_src("lda test:").is_err());

        // op when not expected
        assert!(parse_src("in 12").is_err());

        // op too large
        assert!(parse_src("dat 1234").is_err());
        assert!(parse_src("add 123").is_err());

        // too few ops
        assert!(parse_src("lda").is_err());

        // too many ops
        assert!(parse_src("lda 0 1").is_err());
        assert!(parse_src("lda 0 1 2").is_err());
        assert!(parse_src("lda 0 a_label").is_err());
        assert!(parse_src("lda a_label 0").is_err());
        assert!(parse_src("in 0").is_err());
        assert!(parse_src("in a_label").is_err());
        assert!(parse_src("dat 123 a_label").is_err());
        assert!(parse_src("dat a_label 123").is_err());
        assert!(parse_src("dat 123 456").is_err());
    }
}
