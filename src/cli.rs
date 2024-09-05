use tabled::{Table, Tabled};

use std::{fmt, io};

use crate::interpreter::{Input, Interpreter, LNCInput, Log, Output};
use crate::vec_io::{QueueInput, StackOutput};
use crate::LNCTest;

#[derive(Default)]
struct CLIInput {
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
    let (mem, _) = crate::make_program(source)?;

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
    let (mem, tests) = crate::make_program(source)?;
    let mut results = vec![];

    for test in tests.iter() {
        results.push(run_test(mem, test)?);
    }

    println!("\n--- test results ---");
    println!("{}", Table::new(results).to_string());

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
