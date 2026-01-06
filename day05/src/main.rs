use std::{
    env, fs,
    io::{Write, stdin, stdout},
};

#[derive(PartialEq)]
enum Op {
    ADD,
    MUL,
    IN,
    OUT,
    JNZ,
    JZ,
    LT,
    CMP,
    HLT,
}

struct Cmd {
    op: Op,
    n_operands: usize,
    writes: bool,
}

struct Cpu {
    ip: usize,
    reg: [i64; 8],
    mode: [i64; 8],
}

fn get_input(filename: &str) -> String {
    fs::read_to_string(filename).expect("Failed to open input.")
}

fn get_program(input: String) -> Vec<i64> {
    let mut program: Vec<i64> = Vec::new();

    for num in input.trim().split(",") {
        // println!("{num}");
        program.push(num.parse().expect("failed to parse number"));
    }

    program
}

fn print_prog(program: &[i64], ip: usize) {
    for i in 0..program.len() {
        if i == ip {
            print!("\x1b[31m");
        }
        print!("[{}]\x1b[m", program[i]);
    }
    println!();
}

fn get_cmd(instruction: i64) -> Option<Cmd> {
    let opcode = instruction % 100;
    match opcode {
        1 => Some(Cmd {
            op: Op::ADD,
            n_operands: 3,
            writes: true,
        }),
        2 => Some(Cmd {
            op: Op::MUL,
            n_operands: 3,
            writes: true,
        }),
        3 => Some(Cmd {
            op: Op::IN,
            n_operands: 1,
            writes: true,
        }),
        4 => Some(Cmd {
            op: Op::OUT,
            n_operands: 1,
            writes: false,
        }),
        5 => Some(Cmd {
            op: Op::JNZ,
            n_operands: 2,
            writes: false,
        }),
        6 => Some(Cmd {
            op: Op::JZ,
            n_operands: 2,
            writes: false,
        }),
        7 => Some(Cmd {
            op: Op::LT,
            n_operands: 3,
            writes: true,
        }),
        8 => Some(Cmd {
            op: Op::CMP,
            n_operands: 3,
            writes: true,
        }),
        99 => Some(Cmd {
            op: Op::HLT,
            n_operands: 0,
            writes: false,
        }),
        _ => None,
    }
}

fn get_mode(mode: &mut [i64], instruction: i64, n_operands: usize) {
    let mut digits = instruction / 100;

    for i in 0..n_operands {
        mode[i] = digits % 10;
        digits /= 10;
    }
}

fn read_input() -> i64 {
    print!("\x1b[1;32mINPUT  <\x1b[m ");
    stdout().flush().unwrap();

    let mut input = String::new();

    stdin().read_line(&mut input).expect("Failed to read line");

    input.trim().parse().expect("Failed to read input number")
}

fn execute_cmd(cpu: &mut Cpu, memory: &mut [i64], cmd: Cmd) {
    let boundary = if cmd.writes { 1 } else { 0 };
    for i in 0..cmd.n_operands - boundary {
        match cpu.mode[i] {
            0 => cpu.reg[i] = memory[cpu.reg[i] as usize],
            1 => (),
            _ => (),
        }
    }

    match cmd.op {
        Op::ADD => memory[cpu.reg[2] as usize] = cpu.reg[0] + cpu.reg[1],
        Op::MUL => memory[cpu.reg[2] as usize] = cpu.reg[0] * cpu.reg[1],
        Op::IN => memory[cpu.reg[0] as usize] = read_input(),
        Op::OUT => println!("\x1b[1;31mOUTPUT >\x1b[m {}", cpu.reg[0]),
        Op::JNZ => {
            if cpu.reg[0] != 0 {
                cpu.ip = cpu.reg[1] as usize
            }
        }
        Op::JZ => {
            if cpu.reg[0] == 0 {
                cpu.ip = cpu.reg[1] as usize
            }
        }
        Op::LT => {
            if cpu.reg[0] < cpu.reg[1] {
                memory[cpu.reg[2] as usize] = 1;
            } else {
                memory[cpu.reg[2] as usize] = 0;
            }
        }
        Op::CMP => {
            if cpu.reg[0] == cpu.reg[1] {
                memory[cpu.reg[2] as usize] = 1;
            } else {
                memory[cpu.reg[2] as usize] = 0;
            }
        }
        Op::HLT => (),
    }
}

fn execute_program(program: &Vec<i64>, noun: i64, verb: i64) -> i64 {
    let mut cpu = Cpu {
        ip: 0,
        reg: [0; 8],
        mode: [0; 8],
    };
    let mut memory = program.clone();

    // memory[1] = noun;
    // memory[2] = verb;
    loop {
        // print_prog(&memory, cpu.ip);
        let instruction = memory[cpu.ip];
        let cmd: Cmd = get_cmd(memory[cpu.ip]).expect("Invalid opcode encountered!");
        get_mode(&mut cpu.mode, instruction, cmd.n_operands);

        if cmd.op == Op::HLT {
            break;
        }

        cpu.ip += 1;
        for i in 0..cmd.n_operands {
            cpu.reg[i] = memory[cpu.ip];
            cpu.ip += 1;
            // println!("{}", cpu.reg[i]);
        }

        execute_cmd(&mut cpu, &mut memory, cmd);
    }
    memory[0]
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("no input provided!");
        return;
    }

    let input = get_input(&args[1]);

    let program = get_program(input);
    // print_prog(&program, 0);

    let output = execute_program(&program, 12, 2);

    println!("output: {output}");
}
