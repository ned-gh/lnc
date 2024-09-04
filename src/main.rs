use lnc::cli;

fn main() {
    let source = "
        inp
        loop:
            out         ; this is a comment
            sub one
            sto count
            brp loop
            hlt

        one: dat 1
        count: dat 0";

    if let Err(e) = cli::run(source) {
        println!("{e}");
    }
}
