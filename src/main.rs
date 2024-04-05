use brainfuck::brainfuck as bf;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        panic!("Please provide a brainfuck file");
    }
    let bf_program = match bf::load_program(&args[1]) {
        Ok(program) => program,
        Err(err) => {
            panic!("{}", err);
        }
    };
    bf::run_program(bf_program).unwrap();
}
