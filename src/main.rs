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
        count: dat 0

        .test1 [5] [5, 4, 3, 2, 1, 0]
        .test2 [2] [2, 1, 0, ]";

    if let Err(e) = cli::run_debugger(source) {
        println!("{e}");
    }
}
