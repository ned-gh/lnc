mod assembler;
mod interpreter;
mod lex;
mod parse;
mod vec_io;

pub mod cli;

use parse::LNCTest;

pub fn make_program(source: &str) -> Result<([usize; 100], Vec<LNCTest>), String> {
    let mut errors = vec![];

    let tokens = match lex::tokenize(source) {
        Ok(toks) => toks,
        Err((toks, e)) => {
            errors.push(e);
            toks
        }
    };
    let parse_info = match parse::parse(&tokens) {
        Ok(pi) => pi,
        Err((pi, e)) => {
            errors.push(e);
            pi
        }
    };
    let mem = match assembler::assemble(&parse_info) {
        Ok(m) => m,
        Err(e) => {
            errors.push(e);
            return Err(errors.join("\n"));
        }
    };

    if !errors.is_empty() {
        Err(errors.join("\n"))
    } else {
        Ok((mem, parse_info.tests))
    }
}
