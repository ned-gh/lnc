use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenKind {
    Number(usize),
    Label(String),
    LabelDef(String),
    Load,
    Store,
    Add,
    Subtract,
    Input,
    Output,
    Halt,
    BranchZero,
    BranchPositive,
    BranchAlways,
    Data,
    NewLine,
    Eof,

    // for tests
    TestName(String),
    OpenSquareBracket,
    CloseSquareBracket,
    Comma,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
}

struct Lexer<'a> {
    source: &'a str,
    it: Peekable<Chars<'a>>,
    tokens: Vec<Token>,
    start: usize,
    pos: usize,
    line: usize,
}

impl<'a> Lexer<'a> {
    fn new(line: usize, source: &'a str) -> Self {
        Self {
            source,
            it: source.chars().peekable(),
            tokens: vec![],
            start: 0,
            pos: 0,
            line,
        }
    }

    fn make_err_msg(&self, msg: String) -> String {
        format!("error @ line {}: {}", self.line, msg)
    }

    fn make_tokens(mut self) -> Result<Vec<Token>, String> {
        while let Some(ch) = self.consume() {
            match ch {
                ';' => break,
                '.' => self.test_name()?,
                '[' => self.add_token(TokenKind::OpenSquareBracket),
                ']' => self.add_token(TokenKind::CloseSquareBracket),
                ',' => self.add_token(TokenKind::Comma),
                ch if ch.is_whitespace() => (),
                ch if ch.is_ascii_digit() => self.number()?,
                ch if ch.is_ascii_alphabetic() => self.kw_or_label()?,
                _ => return Err(self.make_err_msg(format!("unexpected character '{}'", ch))),
            }

            self.start = self.pos;
        }

        self.add_token(TokenKind::NewLine);

        Ok(self.tokens)
    }

    fn consume(&mut self) -> Option<char> {
        self.pos += 1;
        self.it.next()
    }

    fn peek(&mut self) -> Option<&char> {
        self.it.peek()
    }

    fn lexeme(&self) -> String {
        self.source[self.start..self.pos].to_owned()
    }

    fn add_token(&mut self, kind: TokenKind) {
        let token = Token {
            kind,
            line: self.line,
        };
        self.tokens.push(token);
    }

    fn consume_while<F>(&mut self, condition: F)
    where
        F: Fn(&char) -> bool,
    {
        while let Some(ch) = self.peek() {
            if condition(ch) {
                self.consume();
            } else {
                break;
            }
        }
    }

    fn number(&mut self) -> Result<(), String> {
        self.consume_while(|ch| ch.is_ascii_digit());

        match self.lexeme().parse::<usize>() {
            Ok(n) => self.add_token(TokenKind::Number(n)),
            Err(_) => {
                return Err(
                    self.make_err_msg(format!("invalid number literal \"{}\"", self.lexeme()))
                )
            }
        }

        Ok(())
    }

    fn kw_or_label(&mut self) -> Result<(), String> {
        self.consume_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_');

        let lexeme = self.lexeme();
        let is_label_def = matches!(self.peek(), Some(':'));

        if let Some(kind) = map_kw(&lexeme) {
            if is_label_def {
                return Err(
                    self.make_err_msg(format!("cannot use keyword \"{lexeme}\" as label name"))
                );
            }
            self.add_token(kind);
        } else if is_label_def {
            self.add_token(TokenKind::LabelDef(lexeme));
            self.consume();
        } else {
            self.add_token(TokenKind::Label(lexeme));
        }

        Ok(())
    }

    fn test_name(&mut self) -> Result<(), String> {
        if let Some(ch) = self.consume() {
            if !(ch.is_ascii_alphabetic() || ch == '_') {
                return Err(format!(
                    "unexpected character '{ch}': test names must start with a letter or '_'"
                ));
            }
        } else {
            return Err("tests must have a name".into());
        }
        self.consume_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_');

        self.start += 1;
        let lexeme = self.lexeme();

        self.add_token(TokenKind::TestName(lexeme));

        Ok(())
    }
}

fn map_kw(word: &str) -> Option<TokenKind> {
    match word {
        "lda" => Some(TokenKind::Load),
        "sto" => Some(TokenKind::Store),
        "add" => Some(TokenKind::Add),
        "sub" => Some(TokenKind::Subtract),
        "inp" => Some(TokenKind::Input),
        "out" => Some(TokenKind::Output),
        "hlt" => Some(TokenKind::Halt),
        "brz" => Some(TokenKind::BranchZero),
        "brp" => Some(TokenKind::BranchPositive),
        "bra" => Some(TokenKind::BranchAlways),
        "dat" => Some(TokenKind::Data),
        _ => None,
    }
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, (Vec<Token>, String)> {
    let mut tokens = vec![];
    let mut errors = vec![];

    for (i, line) in source.lines().enumerate() {
        let lexer = Lexer::new(i, line);

        match lexer.make_tokens() {
            Ok(t) => tokens.extend(t.into_iter()),
            Err(e) => errors.push(e),
        }
    }

    if errors.is_empty() {
        tokens.push(Token {
            kind: TokenKind::Eof,
            line: source.lines().count(),
        });
        Ok(tokens)
    } else {
        Err((tokens, errors.join("\n")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single(source: &str) -> TokenKind {
        tokenize(source).unwrap().remove(0).kind
    }

    #[test]
    fn tokenize_kw() {
        assert_eq!(single("lda"), TokenKind::Load);
        assert_eq!(single("sto"), TokenKind::Store);
        assert_eq!(single("add"), TokenKind::Add);
        assert_eq!(single("sub"), TokenKind::Subtract);
        assert_eq!(single("inp"), TokenKind::Input);
        assert_eq!(single("out"), TokenKind::Output);
        assert_eq!(single("hlt"), TokenKind::Halt);
        assert_eq!(single("brz"), TokenKind::BranchZero);
        assert_eq!(single("brp"), TokenKind::BranchPositive);
        assert_eq!(single("bra"), TokenKind::BranchAlways);
        assert_eq!(single("dat"), TokenKind::Data);
    }

    #[test]
    fn tokenize_label() {
        assert_eq!(single("test_label"), TokenKind::Label("test_label".into()));
        assert_eq!(
            single("test_label:"),
            TokenKind::LabelDef("test_label".into())
        );
        assert_eq!(single("HasCaps"), TokenKind::Label("HasCaps".into()));
        assert_eq!(single("HasNums123"), TokenKind::Label("HasNums123".into()));
    }

    #[test]
    fn no_kw_as_labeldef() {
        assert!(tokenize("lda:").is_err());
        assert!(tokenize("sto:").is_err());
        assert!(tokenize("add:").is_err());
        assert!(tokenize("sub:").is_err());
        assert!(tokenize("inp:").is_err());
        assert!(tokenize("out:").is_err());
        assert!(tokenize("hlt:").is_err());
        assert!(tokenize("brz:").is_err());
        assert!(tokenize("brp:").is_err());
        assert!(tokenize("bra:").is_err());
        assert!(tokenize("dat:").is_err());
    }

    #[test]
    fn tokenize_num() {
        assert_eq!(single("123"), TokenKind::Number(123));
        assert_eq!(single("000123"), TokenKind::Number(123));
        assert!(tokenize("12.3").is_err());
    }

    #[test]
    fn unrecognised_char() {
        assert!(tokenize(":").is_err());
        assert!(tokenize("*").is_err());
        assert!(tokenize("+").is_err());
        assert!(tokenize("-").is_err());
        assert!(tokenize("add 23 ; !@#$%^&*()").is_ok());
    }

    #[test]
    fn tokenize_lnc_test() {
        assert_eq!(
            single(".test_name123"),
            TokenKind::TestName("test_name123".into())
        );
        assert_eq!(single("["), TokenKind::OpenSquareBracket);
        assert_eq!(single("]"), TokenKind::CloseSquareBracket);
        assert_eq!(single(","), TokenKind::Comma);

        assert!(tokenize(".1").is_err());
        assert!(tokenize(".1test").is_err());
    }
}
