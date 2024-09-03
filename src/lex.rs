use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
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
}

struct Lexer<'a> {
    source: &'a str,
    it: Peekable<Chars<'a>>,
    tokens: Vec<Token>,
    start: usize,
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            it: source.chars().peekable(),
            tokens: vec![],
            start: 0,
            pos: 0,
        }
    }

    fn make_tokens(mut self) -> Result<Vec<Token>, String> {
        while let Some(ch) = self.consume() {
            match ch {
                ';' => break,
                ch if ch.is_whitespace() => (),
                ch if ch.is_ascii_digit() => self.number()?,
                ch if ch.is_ascii_alphabetic() => self.kw_or_label()?,
                _ => return Err(format!("Unrecognised character '{ch}'")),
            }

            self.start = self.pos;
        }
        
        self.add_token(Token::NewLine);

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

    fn add_token(&mut self, token: Token) {
        self.tokens.push(token);
    }

    fn consume_while<F>(&mut self, condition: F)
        where F: Fn(&char) -> bool
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
            Ok(n) => self.add_token(Token::Number(n)),
            Err(_) => return Err(format!("Invalid number literal '{}'", self.lexeme())),
        }

        Ok(())
    }

    fn kw_or_label(&mut self) -> Result<(), String> {
        self.consume_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_');

        let lexeme = self.lexeme();

        if let Some(token) = map_kw(&lexeme) {
            self.add_token(token);
            return Ok(());
        }

        if matches!(self.peek(), Some(':')) {
            self.add_token(Token::LabelDef(lexeme));
            self.consume();
        } else {
            self.add_token(Token::Label(lexeme));
        }

        Ok(())
    }
}

fn map_kw(word: &str) -> Option<Token> {
    match word {
        "lda" => Some(Token::Load),
        "sto" => Some(Token::Store),
        "add" => Some(Token::Add),
        "sub" => Some(Token::Subtract),
        "inp" => Some(Token::Input),
        "out" => Some(Token::Output),
        "hlt" => Some(Token::Halt),
        "brz" => Some(Token::BranchZero),
        "brp" => Some(Token::BranchPositive),
        "bra" => Some(Token::BranchAlways),
        "dat" => Some(Token::Data),
        _ => None
    }
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = vec![];
    let mut errors = vec![];

    for (i, line) in source.lines().enumerate() {
        let lexer = Lexer::new(line);

        match lexer.make_tokens() {
            Ok(t) => tokens.extend(t.into_iter()),
            Err(e) => errors.push(format!("Error {i}: {e}")),
        }
    }

    if errors.is_empty() {
        tokens.push(Token::Eof);
        Ok(tokens)
    } else {
        Err(errors.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single(source: &str) -> Token {
        tokenize(source).unwrap().remove(0)
    }

    #[test]
    fn tokenize_kw() {
        assert_eq!(single("lda"), Token::Load);
        assert_eq!(single("sto"), Token::Store);
        assert_eq!(single("add"), Token::Add);
        assert_eq!(single("sub"), Token::Subtract);
        assert_eq!(single("inp"), Token::Input);
        assert_eq!(single("out"), Token::Output);
        assert_eq!(single("hlt"), Token::Halt);
        assert_eq!(single("brz"), Token::BranchZero);
        assert_eq!(single("brp"), Token::BranchPositive);
        assert_eq!(single("bra"), Token::BranchAlways);
        assert_eq!(single("dat"), Token::Data);
    }

    #[test]
    fn tokenize_label() {
        assert_eq!(single("test_label"), Token::Label("test_label".into()));
        assert_eq!(single("test_label:"), Token::LabelDef("test_label".into()));
        assert_eq!(single("HasCaps"), Token::Label("HasCaps".into()));
        assert_eq!(single("HasNums123"), Token::Label("HasNums123".into()));
    }

    #[test]
    fn tokenize_num() {
        assert_eq!(single("123"), Token::Number(123));
        assert_eq!(single("000123"), Token::Number(123));
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
}
