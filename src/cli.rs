use std::io;

use crate::{
    assembler,
    interpreter::{Input, Interpreter, LNCInput, Log, Output},
    lex, parse,
};

#[derive(Default)]
pub struct CLIInput {
    history: Vec<usize>,
}

impl Input for CLIInput {
    fn take(&mut self) -> Result<LNCInput, String> {
        let mut input = String::new();
        if let Err(e) = io::stdin().read_line(&mut input) {
            return Err(format!("Error: {e:?}"));
        }

        let num = match input.trim().parse::<usize>() {
            Ok(n) => n,
            Err(e) => return Err(format!("Error with input \"{}\": {e:?}", input.trim())),
        };

        let maybe_lnc_num = LNCInput::new(num);

        match maybe_lnc_num {
            Some(lnc_num) => {
                self.history.push(num);
                Ok(lnc_num)
            }
            None => Err("Error: input too large".into()),
        }
    }
}

#[derive(Default)]
pub struct CLIOutput {
    history: Vec<usize>,
}

impl Output for CLIOutput {
    fn send(&mut self, val: usize) {
        self.history.push(val);
        println!("Output: {val}");
    }
}

pub struct CLILogger;

impl Log for CLILogger {
    fn log(&mut self, msg: String) {
        println!("{msg}");
    }
}

pub fn run(source: &str) -> Result<(), String> {
    let tokens = lex::tokenize(source)?;
    let parse_info = parse::parse(&tokens)?;
    let mem = assembler::assemble(&parse_info)?;

    let mut input = CLIInput::default();
    let mut output = CLIOutput::default();
    let mut logger = CLILogger;

    let mut interpreter = Interpreter::new(mem, &mut input, &mut output, &mut logger);
    let mut ins_count = 0;

    while !interpreter.is_halted() {
        interpreter.step()?;
        ins_count += 1;
    }

    println!("--- summary ---");
    println!("instruction count: {ins_count}");
    println!("in:  {:?}", input.history);
    println!("out: {:?}", output.history);

    Ok(())
}
