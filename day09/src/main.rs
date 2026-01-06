use core::panic;
use std::{
    collections::VecDeque,
    env, fs,
    io::{Write, stdin, stdout},
    process::{Output, exit},
};

#[derive(PartialEq, Debug)]
enum Op {
    Add,
    Mul,
    In,
    Out,
    Jnz,
    Jz,
    Lt,
    Cmp,
    AdjBp,
    Hlt,
}

#[derive(Default)]
enum CpuMode {
    #[default]
    Normal,
    BreakOnOutput,
}

#[derive(Copy, Clone)]
enum RegMode {
    Pos,
    Imm,
    Rel,
}

#[derive(Default)]
enum State {
    Active,
    Ready,
    #[default]
    Halted,
}

struct Cmd {
    op: Op,
    n_operands: usize,
    writes: bool,
}

struct Cpu {
    ip: usize,
    bp: i64,
    reg: [i64; 8],
    reg_mode: [RegMode; 8],
    memory: Vec<i64>,
    io_in: VecDeque<i64>,
    io_out: VecDeque<i64>,
    mode: CpuMode,
    state: State,
}

impl Cpu {
    fn new() -> Self {
        let mut new = Self {
            ip: 0,
            bp: 0,
            reg: [0; 8],
            reg_mode: [RegMode::Pos; 8],
            memory: Vec::new(),
            io_in: VecDeque::new(),
            io_out: VecDeque::new(),
            mode: CpuMode::Normal,
            state: State::Halted,
        };
        new.memory.resize(1_000_000, 0);
        new
    }

    fn load_program(&mut self, program: &[i64]) {
        self.ip = 0;
        self.bp = 0;
        self.io_in.clear();
        self.io_out.clear();
        self.state = State::Ready;
        self.memory.fill(0);
        self.memory[0..program.len()].copy_from_slice(program);
    }

    fn print_cmd(&self, cmd: &Cmd) {
        print!(
            "\x1b[33m{:4}\x1b[m : \x1b[34m{:4}\x1b[m   ",
            self.bp, self.ip
        );
        print!("\x1b[31m{:?}\x1b[m\t", cmd.op);
        for i in 0..=cmd.n_operands {
            print!("[{}]", self.memory[self.ip + i]);
        }
        println!();
    }

    fn run(&mut self) {
        self.state = State::Active;
        loop {
            // print_prog(&self.memory, self.ip);
            let instruction = self.memory[self.ip];
            let cmd: Cmd = get_cmd(self.memory[self.ip]).expect("Invalid opcode encountered!");
            get_mode(&mut self.reg_mode, instruction, cmd.n_operands);
            self.print_cmd(&cmd);

            for i in 0..cmd.n_operands {
                self.reg[i] = self.memory[self.ip + i + 1];
                // println!("{}", cpu.reg[i]);
            }

            self.ip += cmd.n_operands + 1;
            execute_cmd(self, cmd);

            let State::Active = self.state else {
                break;
            };
        }
    }
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

fn dump_program(program: &[i64]) {
    for (i, num) in program.iter().enumerate() {
        println!("{i} : {num}");
    }
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
            op: Op::Add,
            n_operands: 3,
            writes: true,
        }),
        2 => Some(Cmd {
            op: Op::Mul,
            n_operands: 3,
            writes: true,
        }),
        3 => Some(Cmd {
            op: Op::In,
            n_operands: 1,
            writes: true,
        }),
        4 => Some(Cmd {
            op: Op::Out,
            n_operands: 1,
            writes: false,
        }),
        5 => Some(Cmd {
            op: Op::Jnz,
            n_operands: 2,
            writes: false,
        }),
        6 => Some(Cmd {
            op: Op::Jz,
            n_operands: 2,
            writes: false,
        }),
        7 => Some(Cmd {
            op: Op::Lt,
            n_operands: 3,
            writes: true,
        }),
        8 => Some(Cmd {
            op: Op::Cmp,
            n_operands: 3,
            writes: true,
        }),
        9 => Some(Cmd {
            op: Op::AdjBp,
            n_operands: 1,
            writes: false,
        }),
        99 => Some(Cmd {
            op: Op::Hlt,
            n_operands: 0,
            writes: false,
        }),
        _ => None,
    }
}

fn get_mode(mode: &mut [RegMode], instruction: i64, n_operands: usize) {
    let mut digits = instruction / 100;

    for i in 0..n_operands {
        mode[i] = match digits % 10 {
            0 => RegMode::Pos,
            1 => RegMode::Imm,
            2 => RegMode::Rel,
            _ => panic!("Register mode not implemented!"),
        };
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

fn execute_cmd(cpu: &mut Cpu, cmd: Cmd) {
    let boundary = if cmd.writes { 1 } else { 0 };
    for i in 0..cmd.n_operands - boundary {
        match cpu.reg_mode[i] {
            RegMode::Pos => cpu.reg[i] = cpu.memory[cpu.reg[i] as usize],
            RegMode::Imm => (),
            RegMode::Rel => cpu.reg[i] = cpu.memory[(cpu.bp + cpu.reg[i]) as usize],
        }
    }

    match cmd.op {
        Op::Add => {
            if let RegMode::Rel = cpu.reg_mode[2] {
                cpu.reg[2] += cpu.bp;
            }
            cpu.memory[cpu.reg[2] as usize] = cpu.reg[0] + cpu.reg[1]
        }
        Op::Mul => {
            if let RegMode::Rel = cpu.reg_mode[2] {
                cpu.reg[2] += cpu.bp;
            }
            cpu.memory[cpu.reg[2] as usize] = cpu.reg[0] * cpu.reg[1]
        }
        Op::In => {
            let input = cpu.io_in.pop_back().expect("No io available to read!");
            if let RegMode::Rel = cpu.reg_mode[0] {
                cpu.reg[0] += cpu.bp;
            }
            cpu.memory[cpu.reg[0] as usize] = input;
            println!("\x1b[1;32mINPUT  <\x1b[m {}", input);
        }
        Op::Out => {
            println!("\x1b[1;31mOUTPUT >\x1b[m {}", cpu.reg[0]);
            cpu.io_out.push_front(cpu.reg[0]);
            if let CpuMode::BreakOnOutput = cpu.mode {
                cpu.state = State::Ready;
            }
        }
        Op::Jnz => {
            if cpu.reg[0] != 0 {
                cpu.ip = cpu.reg[1] as usize
            }
        }
        Op::Jz => {
            if cpu.reg[0] == 0 {
                cpu.ip = cpu.reg[1] as usize
            }
        }
        Op::Lt => {
            if let RegMode::Rel = cpu.reg_mode[2] {
                cpu.reg[2] += cpu.bp;
            }
            if cpu.reg[0] < cpu.reg[1] {
                cpu.memory[cpu.reg[2] as usize] = 1;
            } else {
                cpu.memory[cpu.reg[2] as usize] = 0;
            }
        }
        Op::Cmp => {
            if let RegMode::Rel = cpu.reg_mode[2] {
                cpu.reg[2] += cpu.bp;
            }
            if cpu.reg[0] == cpu.reg[1] {
                cpu.memory[cpu.reg[2] as usize] = 1;
            } else {
                cpu.memory[cpu.reg[2] as usize] = 0;
            }
        }
        Op::AdjBp => cpu.bp += cpu.reg[0],
        Op::Hlt => cpu.state = State::Halted,
    }
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
    // dump_program(&program);
    // exit(0);

    let mut cpu = Cpu::new();

    cpu.load_program(&program);
    cpu.io_in.push_front(2);
    cpu.run();

    let output = cpu.io_out.pop_back().expect("No output!");

    println!("output: {output}");
}
