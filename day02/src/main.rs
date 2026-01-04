use std::{env, fs};

enum State {
    Cmd,
    Src1,
    Src2,
    Dst,
}

enum Ops {
    Add,
    Mult,
    Halt,
}

fn get_input(filename: &str) -> String {
    fs::read_to_string(filename).expect("Failed to open input.")
}

fn parse_ops(input: String) -> Vec<i64> {
    let mut ops: Vec<i64> = Vec::new();

    for num in input.trim().split(",") {
        // println!("{num}");
        ops.push(num.parse().expect("failed to parse number"));
    }

    ops
}

fn print_prog(ops: &[i64]) {
    for op in ops {
        print!("[{op}]");
    }
    println!();
}

fn execute(program: &Vec<i64>, input1: i64, input2: i64) -> i64 {
    let mut memory = program.clone();
    let mut state = State::Cmd;
    let mut cmd = Ops::Halt;
    let mut val1: i64 = 0;
    let mut val2: i64 = 0;

    memory[1] = input1;
    memory[2] = input2;

    for i in 0..memory.len() {
        let num = memory[i];
        match state {
            State::Cmd => {
                match num {
                    1 => cmd = Ops::Add,
                    2 => cmd = Ops::Mult,
                    99 => return memory[0],
                    _ => panic!("Invalid op encountered!"),
                }
                state = State::Src1
            }
            State::Src1 => {
                val1 = memory[num as usize];
                state = State::Src2;
            }
            State::Src2 => {
                val2 = memory[num as usize];
                state = State::Dst;
            }
            State::Dst => {
                match cmd {
                    Ops::Add => memory[num as usize] = val1 + val2,
                    Ops::Mult => memory[num as usize] = val1 * val2,
                    _ => panic!("memory tried to perform halt on operands!"),
                }
                state = State::Cmd
            }
        }
    }

    memory[0]
}

fn find_inputs(program: &Vec<i64>) -> Option<(i64, i64)> {
    for x in 0..100 {
        for y in 0..100 {
            let answer = execute(program, x, y);
            if answer == 19690720 {
                return Some((x, y));
            }
        }
    }

    None
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("no input provided!");
        return;
    }

    let input = get_input(&args[1]);

    let program = parse_ops(input);

    print_prog(&program);

    let inputs: (i64, i64) =
        find_inputs(&program).expect("No valid inputs to produce desired output");

    println!("inputs: {} {}", inputs.0, inputs.1);
    println!("answer: {}", 100 * inputs.0 + inputs.1);
}
