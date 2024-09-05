use std::collections::HashMap;
use std::iter::Peekable;
use std::slice::Iter;

use crate::lex::{Token, TokenKind};

#[derive(Debug, PartialEq, Eq)]
pub enum Address {
    Symbolic(String),
    Numeric(usize),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
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
pub struct LNCTest {
    name: String,
    inputs: Vec<usize>,
    outputs: Vec<usize>,
}

#[derive(Debug)]
pub struct ParseInfo {
    pub instructions: Vec<Instruction>,
    pub label_map: HashMap<String, usize>,
    pub tests: Vec<LNCTest>,
}

impl ParseInfo {
    fn new() -> Self {
        Self {
            instructions: vec![],
            label_map: HashMap::new(),
            tests: vec![],
        }
    }
}

struct Parser<'a> {
    it: Peekable<Iter<'a, Token>>,
    paddr: usize,
    info: ParseInfo,
    errors: Vec<String>,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self {
            it: tokens.iter().peekable(),
            paddr: 0,
            info: ParseInfo::new(),
            errors: vec![],
        }
    }

    fn add_err_msg(&mut self, line: usize, msg: String) {
        self.errors.push(format!("error @ line {}: {}", line, msg));
    }

    fn make_instructions(mut self) -> Result<ParseInfo, (ParseInfo, String)> {
        while let Some(token) = self.consume() {
            let res = match token.kind {
                TokenKind::LabelDef(s) => {
                    self.info.label_map.insert(s, self.paddr);
                    Ok(())
                }
                TokenKind::Load
                | TokenKind::Store
                | TokenKind::Add
                | TokenKind::Subtract
                | TokenKind::BranchZero
                | TokenKind::BranchPositive
                | TokenKind::BranchAlways => self.ins_with_addr(&token),
                TokenKind::Input | TokenKind::Output | TokenKind::Halt => {
                    self.ins_without_addr(&token)
                }
                TokenKind::Data => self.data(),
                TokenKind::NewLine => Ok(()),
                TokenKind::Eof => break,
                TokenKind::Number(n) => Err(format!(
                    "found number ({n}) instead of instruction/label def"
                )),
                TokenKind::Label(s) => Err(format!(
                    "found label \"{s}\" instead of instruction/label def"
                )),
                TokenKind::TestName(s) => self.lnc_test(s),
                TokenKind::OpenSquareBracket => Err("unexpected bracket '['".into()),
                TokenKind::CloseSquareBracket => Err("unexpected bracket ']'".into()),
                TokenKind::Comma => Err("unexpected comma ','".into()),
            };

            if let Err(e) = res {
                self.add_err_msg(token.line, e);
                self.sync();
            }
        }

        if self.errors.is_empty() {
            Ok(self.info)
        } else {
            Err((self.info, self.errors.join("\n")))
        }
    }

    fn sync(&mut self) {
        while let Some(token) = self.peek() {
            if matches!(token.kind, TokenKind::NewLine | TokenKind::Eof) {
                break;
            }
            self.consume();
        }
    }

    fn consume(&mut self) -> Option<Token> {
        self.it.next().cloned()
    }

    fn peek(&mut self) -> Option<&Token> {
        self.it.peek().copied()
    }

    fn add_ins(&mut self, ins: Instruction) {
        self.info.instructions.push(ins);
        self.paddr += 1;
    }

    fn check_next(&mut self, kind: TokenKind) -> Result<(), String> {
        if let Some(next) = self.peek() {
            if next.kind != kind {
                return Err(format!("expected {:?}: found {:?}", kind, next.kind));
            }
        } else {
            return Err(format!("unexpected EOF: expected {:?}", kind));
        }

        self.consume();

        Ok(())
    }

    fn check_newline(&mut self) -> Result<(), String> {
        if let Some(nl_token) = self.peek() {
            if !matches!(nl_token.kind, TokenKind::NewLine | TokenKind::Eof) {
                return Err(format!(
                    "invalid token {:?}: expected end of line",
                    nl_token
                ));
            }
        } else {
            return Err("unexpected EOF: expected address".to_owned());
        }

        self.consume();

        Ok(())
    }

    fn ins_with_addr(&mut self, token: &Token) -> Result<(), String> {
        let addr = if let Some(addr_token) = self.consume() {
            match addr_token.kind {
                TokenKind::Number(n) => {
                    if n >= 100 {
                        return Err(format!("invalid address {}: too large", n));
                    }
                    Address::Numeric(n)
                }
                TokenKind::Label(s) => Address::Symbolic(s),
                _ => return Err(format!("invalid token {:?}: expected address", addr_token)),
            }
        } else {
            return Err("unexpected EOF: expected address".to_owned());
        };

        self.check_newline()?;

        match token.kind {
            TokenKind::Load => self.add_ins(Instruction::Load(addr)),
            TokenKind::Store => self.add_ins(Instruction::Store(addr)),
            TokenKind::Add => self.add_ins(Instruction::Add(addr)),
            TokenKind::Subtract => self.add_ins(Instruction::Subtract(addr)),
            TokenKind::BranchZero => self.add_ins(Instruction::BranchZero(addr)),
            TokenKind::BranchPositive => self.add_ins(Instruction::BranchPositive(addr)),
            TokenKind::BranchAlways => self.add_ins(Instruction::BranchAlways(addr)),
            _ => unreachable!(),
        }

        Ok(())
    }

    fn ins_without_addr(&mut self, token: &Token) -> Result<(), String> {
        self.check_newline()?;

        match token.kind {
            TokenKind::Input => self.add_ins(Instruction::Input),
            TokenKind::Output => self.add_ins(Instruction::Output),
            TokenKind::Halt => self.add_ins(Instruction::Halt),
            _ => unreachable!(),
        }

        Ok(())
    }

    fn data(&mut self) -> Result<(), String> {
        let num = if let Some(num_token) = self.consume() {
            if let TokenKind::Number(n) = num_token.kind {
                if n >= 1000 {
                    return Err(format!("invalid data {}: too large", n));
                }
                n
            } else {
                return Err(format!("invalid token {:?}: expected number", num_token));
            }
        } else {
            return Err("io token found".to_owned());
        };

        self.add_ins(Instruction::Data(num));

        Ok(())
    }

    fn lnc_test(&mut self, name: String) -> Result<(), String> {
        let inputs = self.number_list()?;
        let outputs = self.number_list()?;

        self.check_newline()?;

        self.info.tests.push(LNCTest {
            name,
            inputs,
            outputs,
        });

        Ok(())
    }

    fn number_list(&mut self) -> Result<Vec<usize>, String> {
        self.check_next(TokenKind::OpenSquareBracket)?;

        let mut nums = vec![];
        let mut prev_was_num = false;

        while let Some(token) = self.peek() {
            match token.kind {
                TokenKind::Number(n) => {
                    if prev_was_num {
                        return Err(format!("expected ',' or ']': found number ({n})"));
                    }
                    if n >= 1000 {
                        return Err(format!("invalid number {n}: too large"));
                    }
                    nums.push(n);
                    prev_was_num = true;
                }
                TokenKind::Comma => {
                    if !prev_was_num {
                        return Err("unexpected ','".into());
                    }
                    prev_was_num = false;
                }
                TokenKind::CloseSquareBracket => break,
                _ => return Err(format!("expected number, ',', or ']': found {token:?}")),
            }

            self.consume();
        }

        self.check_next(TokenKind::CloseSquareBracket)?;

        Ok(nums)
    }
}

pub fn parse(tokens: &[Token]) -> Result<ParseInfo, (ParseInfo, String)> {
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

    fn parse_src(source: &str) -> Result<ParseInfo, (ParseInfo, String)> {
        let tokens = tokenize(source).unwrap();
        parse(&tokens)
    }

    fn get_nlist(source: &str) -> Result<Vec<usize>, String> {
        let tokens = tokenize(source).unwrap();
        let mut parser = Parser::new(&tokens);
        parser.number_list()
    }

    fn make_test(name: &str, inputs: Vec<usize>, outputs: Vec<usize>) -> LNCTest {
        LNCTest {
            name: name.into(),
            inputs,
            outputs,
        }
    }

    fn get_test(source: &str) -> LNCTest {
        parse_src(source).unwrap().tests.remove(0)
    }

    #[test]
    fn parse_with_addr() {
        use Address::Numeric;

        assert_eq!(single("lda 01"), Instruction::Load(Numeric(1)));
        assert_eq!(single("sto 02"), Instruction::Store(Numeric(2)));
        assert_eq!(single("add 03"), Instruction::Add(Numeric(3)));
        assert_eq!(single("sub 04"), Instruction::Subtract(Numeric(4)));
        assert_eq!(single("brz 05"), Instruction::BranchZero(Numeric(5)));
        assert_eq!(single("brp 06"), Instruction::BranchPositive(Numeric(6)));
        assert_eq!(single("bra 07"), Instruction::BranchAlways(Numeric(7)));
    }

    #[test]
    fn parse_with_symbolic_addr() {
        use Address::Symbolic;

        assert_eq!(
            single("lda this"),
            Instruction::Load(Symbolic("this".into()))
        );
        assert_eq!(single("sto is"), Instruction::Store(Symbolic("is".into())));
        assert_eq!(single("add a"), Instruction::Add(Symbolic("a".into())));
        assert_eq!(
            single("sub test"),
            Instruction::Subtract(Symbolic("test".into()))
        );
        assert_eq!(
            single("brz with"),
            Instruction::BranchZero(Symbolic("with".into()))
        );
        assert_eq!(
            single("brp symbolic"),
            Instruction::BranchPositive(Symbolic("symbolic".into()))
        );
        assert_eq!(
            single("bra addresses"),
            Instruction::BranchAlways(Symbolic("addresses".into()))
        );
    }

    #[test]
    fn parse_without_addr() {
        assert_eq!(single("inp"), Instruction::Input);
        assert_eq!(single("out"), Instruction::Output);
        assert_eq!(single("hlt"), Instruction::Halt);
    }

    #[test]
    fn parse_data() {
        assert_eq!(single("dat 123"), Instruction::Data(123));
    }

    #[test]
    fn parse_handles_newlines() {
        use Address::Numeric;

        let source = "
            lda 10 ; this is
            add 11 ; a comment

            sto 10
            hlt";
        let info = parse_src(source).unwrap();
        let expected = vec![
            Instruction::Load(Numeric(10)),
            Instruction::Add(Numeric(11)),
            Instruction::Store(Numeric(10)),
            Instruction::Halt,
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

    #[test]
    fn maps_label_addr() {
        let src = "
        test:
        another_test:

        this_should_be_0:
        add 1
        add 3
        sub 2

        a_label:
        inp
        b_label:";

        let info = parse_src(src).unwrap();

        let expected = HashMap::from([
            ("test".to_owned(), 0),
            ("another_test".to_owned(), 0),
            ("this_should_be_0".to_owned(), 0),
            ("a_label".to_owned(), 3),
            ("b_label".to_owned(), 4),
        ]);

        assert_eq!(info.label_map, expected);
    }

    #[test]
    fn parse_number_list() {
        assert_eq!(get_nlist("[]").unwrap(), vec![]);
        assert_eq!(get_nlist("[1]").unwrap(), vec![1]);
        assert_eq!(get_nlist("[1,]").unwrap(), vec![1]);
        assert_eq!(get_nlist("[1, 2, 3]").unwrap(), vec![1, 2, 3]);
        assert_eq!(get_nlist("[1,    2, 3]").unwrap(), vec![1, 2, 3]);
        assert_eq!(get_nlist("[1,    2, 3        ]").unwrap(), vec![1, 2, 3]);
        assert_eq!(get_nlist("[1,2,3]").unwrap(), vec![1, 2, 3]);
        assert_eq!(get_nlist("[1, 2, 3, ]").unwrap(), vec![1, 2, 3]);

        assert!(get_nlist("[,]").is_err());
        assert!(get_nlist("[add]").is_err());
        assert!(get_nlist("[label:]").is_err());
        assert!(get_nlist("[label:, 1]").is_err());
        assert!(get_nlist("[1, label:]").is_err());
        assert!(get_nlist("[1234]").is_err());
        assert!(get_nlist("[1, 2, 1234]").is_err());
    }

    #[test]
    fn parse_test() {
        assert_eq!(
            get_test(".test_name [1, 2, 3] [1, 2, 3, 4]"),
            make_test("test_name", vec![1, 2, 3], vec![1, 2, 3, 4]),
        );
        assert_eq!(
            get_test(".test_name [] []"),
            make_test("test_name", vec![], vec![]),
        );
        assert_eq!(
            get_test(".test123 [1, 2, 3] [1, 2, 3]"),
            make_test("test123", vec![1, 2, 3], vec![1, 2, 3]),
        );
        assert_eq!(
            get_test("._test123 [1, 2, 3] [1, 2, 3]"),
            make_test("_test123", vec![1, 2, 3], vec![1, 2, 3]),
        );

        assert!(parse_src(".test_name").is_err());
        assert!(parse_src(".test_name [1, 2, 3]").is_err());
        assert!(parse_src(".test_name [1, 2, 3] [1, 2, 3] [1, 2, 3]").is_err());
    }
}
