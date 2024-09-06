use tabled::{builder::Builder, settings::Style, Table, Tabled};

use std::collections::HashMap;
use std::{fmt, io, io::Write};

use crate::interpreter::{Input, Interpreter, LNCInput, Log, Output};
use crate::vec_io::{QueueInput, StackOutput};
use crate::LNCTest;

#[derive(Default)]
struct CLIInput {
    history: Vec<usize>,
}

impl Input for CLIInput {
    fn take(&mut self) -> Result<LNCInput, String> {
        print!("Enter input value: ");
        let _ = io::stdout().flush();

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
struct CLIOutput {
    history: Vec<usize>,
}

impl Output for CLIOutput {
    fn send(&mut self, val: usize) {
        self.history.push(val);
        println!("Output: {val}");
    }
}

struct CLILogger;

impl Log for CLILogger {
    fn log(&mut self, msg: String) {
        println!("{msg}");
    }
}

enum TestResult {
    Passed,
    Failed(String),
}

impl fmt::Display for TestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Passed => write!(f, "ok"),
            Self::Failed(msg) => write!(f, "failed: {msg}"),
        }
    }
}

#[derive(Tabled)]
struct LNCTestInfo {
    name: String,
    input: String,
    expected_output: String,
    actual_output: String,
    ins_count: usize,
    result: TestResult,
}

impl LNCTestInfo {
    fn new(test: &LNCTest, actual_output: &[usize], ins_count: usize, result: TestResult) -> Self {
        Self {
            name: test.name.to_owned(),
            input: format!("{:?}", test.inputs),
            expected_output: format!("{:?}", test.outputs),
            actual_output: format!("{actual_output:?}"),
            ins_count,
            result,
        }
    }
}

pub fn run(source: &str) -> Result<(), String> {
    let mem = crate::make_program(source)?.mem;

    let mut input = CLIInput::default();
    let mut output = CLIOutput::default();
    let mut logger = CLILogger;

    let mut interpreter = Interpreter::new(mem, &mut input, &mut output, &mut logger);
    let mut ins_count = 0;

    while !interpreter.is_halted() {
        interpreter.step()?;
        ins_count += 1;
    }

    println!("\n--- summary ---");
    println!("instruction count: {ins_count}");
    println!("in:  {:?}", input.history);
    println!("out: {:?}", output.history);

    Ok(())
}

pub fn run_tests(source: &str) -> Result<(), String> {
    let program = crate::make_program(source)?;
    let (mem, tests) = (program.mem, program.parse_info.tests);

    let mut results = vec![];

    for test in tests.iter() {
        results.push(run_test(mem, test)?);
    }

    println!("\n--- test results ---");
    println!("{}", Table::new(results).with(Style::sharp()));

    Ok(())
}

pub fn run_debugger(source: &str) -> Result<(), String> {
    let program = crate::make_program(source)?;

    let mem = program.mem;
    let addr_to_label: HashMap<usize, String> = program
        .parse_info
        .label_map
        .into_iter()
        .map(|(k, v)| (v, k))
        .collect();

    let mut input = CLIInput::default();
    let mut output = CLIOutput::default();
    let mut logger = CLILogger;

    let mut interpreter = Interpreter::new(mem, &mut input, &mut output, &mut logger);
    let mut ins_count = 0;
    let mut skip_count = 0;

    while !interpreter.is_halted() {
        println!("\n--- ins #{ins_count} ---");
        let state = interpreter.state();

        let mut builder = Builder::default();
        builder.push_record(["pc", "addr", "label", "mnemonic", "mem"]);

        let height = 15;

        let (min, max) = if state.pc < height / 2 {
            (0, height - 1)
        } else if state.pc > (99 - height / 2) {
            (99 - height + 1, 99)
        } else {
            (state.pc - height / 2, state.pc + height / 2)
        };

        for (addr, val) in state
            .mem
            .iter()
            .enumerate()
            .filter(|(addr, _)| *addr >= min && *addr <= max)
        {
            let arrow = if addr == state.pc { ">" } else { "" };
            let addr_str = format!("{addr:02}");
            let label = if let Some(l) = addr_to_label.get(&addr) {
                l
            } else {
                ""
            };

            let first_digit = val / 100;
            let op = val % 100;
            let mnemonic = match first_digit {
                5 => format!("lda {:02}", op),
                3 => format!("sto {:02}", op),
                1 => format!("add {:02}", op),
                2 => format!("sub {:02}", op),
                9 => match op {
                    01 => "inp".to_owned(),
                    02 => "out".to_owned(),
                    _ => "".to_owned(),
                },
                0 => {
                    if op == 0 {
                        "hlt".to_owned()
                    } else {
                        "".to_owned()
                    }
                }
                7 => format!("brz {:02}", op),
                8 => format!("brp {:02}", op),
                6 => format!("bra {:02}", op),
                _ => "".to_owned(),
            };
            let val_str = format!("{:03}", val);

            builder.push_record([arrow, &addr_str, label, &mnemonic, &val_str]);
        }

        let mem_table = builder.build().with(Style::sharp()).to_string();
        println!("{mem_table}");

        let mut builder = Builder::default();

        builder.push_record(["pc", "acc", "neg_flag", "halted"]);
        builder.push_record([
            state.pc.to_string(),
            state.acc.to_string(),
            state.neg_flag.to_string(),
            state.halted.to_string(),
        ]);

        let state_table = builder.build().with(Style::sharp()).to_string();
        println!("{state_table}");

        if skip_count == 0 {
            loop {
                print!(">>> ");
                let _ = io::stdout().flush();

                let mut input = String::new();

                if io::stdin().read_line(&mut input).is_err() {
                    continue;
                }

                let input = input.trim();

                if input.is_empty() {
                    skip_count = 1;
                    break;
                }

                match input.parse::<usize>() {
                    Ok(n) => {
                        skip_count = n.max(1);
                        break;
                    }
                    Err(_) => continue,
                };
            }
        }

        interpreter.step()?;
        ins_count += 1;
        skip_count -= 1;
    }

    let mut builder = Builder::default();
    builder.push_record(["ins_count", "in", "out"]);
    builder.push_record([
        ins_count.to_string(),
        format!("{:?}", input.history),
        format!("{:?}", output.history),
    ]);

    let result_table = builder.build().with(Style::sharp()).to_string();

    println!("\n--- summary ---");
    println!("{result_table}");

    Ok(())
}

fn run_test(mem: [usize; 100], test: &LNCTest) -> Result<LNCTestInfo, String> {
    let mut input = QueueInput::new(&test.inputs)?;
    let mut output = StackOutput::default();
    let mut logger = CLILogger;

    let mut interpreter = Interpreter::new(mem, &mut input, &mut output, &mut logger);
    let mut ins_count = 0;

    while !interpreter.is_halted() {
        match interpreter.step() {
            Ok(_) => (),
            Err(e) => {
                return Ok(LNCTestInfo::new(
                    test,
                    &output.stack,
                    ins_count,
                    TestResult::Failed(e),
                ));
            }
        }
        ins_count += 1;
    }

    if !input.queue.is_empty() {
        return Ok(LNCTestInfo::new(
            test,
            &output.stack,
            ins_count,
            TestResult::Failed(format!("unused inputs: {:?}", input.queue)),
        ));
    }

    if output.stack != test.outputs {
        return Ok(LNCTestInfo::new(
            test,
            &output.stack,
            ins_count,
            TestResult::Failed("incorrect outputs".into()),
        ));
    }

    Ok(LNCTestInfo::new(
        test,
        &output.stack,
        ins_count,
        TestResult::Passed,
    ))
}
