use std::error::Error;
use std::fs;
use std::path::PathBuf;

use clap::Parser;

use lnc::cli;

#[derive(Parser)]
struct Args {
    /// path to .lmn source code file
    path: PathBuf,

    /// run tests
    #[arg(short, long)]
    test: bool,

    /// run debugger
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let source = fs::read_to_string(args.path)?;

    if args.test {
        if let Err(e) = cli::run_tests(&source) {
            println!("{e}");
        }

        return Ok(());
    }

    if args.debug {
        if let Err(e) = cli::run_debugger(&source) {
            println!("{e}");
        }

        return Ok(());
    }

    if let Err(e) = cli::run(&source) {
        println!("{e}");
    }

    Ok(())
}
