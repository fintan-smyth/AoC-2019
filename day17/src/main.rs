use core::panic;
use std::{
    collections::{HashMap, VecDeque},
    env, fs,
    hash::Hash,
    io::{Read, Write, stdin, stdout},
    thread::sleep,
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode, read},
    terminal,
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

enum Dir {
    North,
    South,
    East,
    West,
}

#[derive(Default)]
enum CpuMode {
    #[default]
    Normal,
    ReadStdin,
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

    fn get_mode(&mut self, instruction: i64, n_operands: usize) {
        let mut digits = instruction / 100;

        let mode: &mut [RegMode] = &mut self.reg_mode;
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

    fn execute_cmd(&mut self, cmd: Cmd) {
        let boundary = if cmd.writes { 1 } else { 0 };
        for i in 0..cmd.n_operands - boundary {
            match self.reg_mode[i] {
                RegMode::Pos => self.reg[i] = self.memory[self.reg[i] as usize],
                RegMode::Imm => (),
                RegMode::Rel => self.reg[i] = self.memory[(self.bp + self.reg[i]) as usize],
            }
        }

        match cmd.op {
            Op::Add => {
                if let RegMode::Rel = self.reg_mode[2] {
                    self.reg[2] += self.bp;
                }
                self.memory[self.reg[2] as usize] = self.reg[0] + self.reg[1]
            }
            Op::Mul => {
                if let RegMode::Rel = self.reg_mode[2] {
                    self.reg[2] += self.bp;
                }
                self.memory[self.reg[2] as usize] = self.reg[0] * self.reg[1]
            }
            Op::In => {
                let input: i64;
                if let CpuMode::ReadStdin = self.mode {
                    input = read_input();
                } else {
                    if self.io_in.is_empty() {
                        self.state = State::Ready;
                        println!("\x1b[35;1mWaiting for IO in...\x1b[m");
                        return;
                    }
                    input = self.io_in.pop_back().expect("No io available to read!");
                    println!("\x1b[1;32mINPUT  <\x1b[m {}", input);
                }
                if let RegMode::Rel = self.reg_mode[0] {
                    self.reg[0] += self.bp;
                }
                self.memory[self.reg[0] as usize] = input;
            }
            Op::Out => {
                println!("\x1b[1;31mOUTPUT >\x1b[m {}", self.reg[0]);
                self.io_out.push_front(self.reg[0]);
            }
            Op::Jnz => {
                if self.reg[0] != 0 {
                    self.ip = self.reg[1] as usize;
                    return;
                }
            }
            Op::Jz => {
                if self.reg[0] == 0 {
                    self.ip = self.reg[1] as usize;
                    return;
                }
            }
            Op::Lt => {
                if let RegMode::Rel = self.reg_mode[2] {
                    self.reg[2] += self.bp;
                }
                if self.reg[0] < self.reg[1] {
                    self.memory[self.reg[2] as usize] = 1;
                } else {
                    self.memory[self.reg[2] as usize] = 0;
                }
            }
            Op::Cmp => {
                if let RegMode::Rel = self.reg_mode[2] {
                    self.reg[2] += self.bp;
                }
                if self.reg[0] == self.reg[1] {
                    self.memory[self.reg[2] as usize] = 1;
                } else {
                    self.memory[self.reg[2] as usize] = 0;
                }
            }
            Op::AdjBp => self.bp += self.reg[0],
            Op::Hlt => {
                self.state = State::Halted;
                return;
            }
        }
        self.ip += cmd.n_operands + 1;
    }

    fn run(&mut self) {
        self.state = State::Active;
        loop {
            // print_prog(&self.memory, self.ip);
            let instruction = self.memory[self.ip];
            let cmd: Cmd = get_cmd(self.memory[self.ip]).expect("Invalid opcode encountered!");
            self.get_mode(instruction, cmd.n_operands);
            // self.print_cmd(&cmd);

            for i in 0..cmd.n_operands {
                self.reg[i] = self.memory[self.ip + i + 1];
                // println!("{}", cpu.reg[i]);
            }

            self.execute_cmd(cmd);

            let State::Active = self.state else {
                break;
            };
        }
    }
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

fn read_input() -> i64 {
    print!("\x1b[1;32mINPUT  <\x1b[m ");
    stdout().flush().unwrap();

    let mut input = [0u8; 1];

    terminal::enable_raw_mode().expect("Failed to enter raw mode");
    stdin().read_exact(&mut input).expect("Failed to read char");
    terminal::disable_raw_mode().expect("Failed to exit raw mode");
    println!();

    let input = input[0] as char;
    match input {
        'a' => -1,
        'd' => 1,
        ' ' => 2,
        _ => 0,
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

fn print_canvas(canvas: &Vec<Vec<char>>) {
    for row in canvas {
        for c in row {
            match c {
                '#' => print!("\x1b[34m"),
                '^' => print!("\x1b[31m"),
                'v' => print!("\x1b[31m"),
                '<' => print!("\x1b[31m"),
                '>' => print!("\x1b[31m"),
                _ => (),
            }
            print!("{c}\x1b[m");
        }
        println!();
    }
}

fn get_alignment_params(view: &[Vec<char>]) -> usize {
    let mut alignment = 0;

    for y in 1..(view.len() - 1) {
        for x in 1..(view[y].len() - 1) {
            // println!("({},{})", x, y);
            if view[y][x] == '#'
                && view[y + 1][x] == '#'
                && view[y - 1][x] == '#'
                && view[y][x + 1] == '#'
                && view[y][x - 1] == '#'
            {
                alignment += x * y;
            }
        }
    }

    alignment
}

fn program_robot(cpu: &mut Cpu) {
    let sub_a = "R,12,L,10,R,12\n";
    let sub_b = "L,8,R,10,R,6\n";
    let sub_c = "R,12,L,10,R,10,L,8\n";
    let routine = "A,B,A,C,B,C,B,C,A,C\n";

    cpu.memory[0] = 2;
    for c in routine.chars() {
        cpu.io_in.push_front(c as u8 as i64);
    }
    for c in sub_a.chars() {
        cpu.io_in.push_front(c as u8 as i64);
    }
    for c in sub_b.chars() {
        cpu.io_in.push_front(c as u8 as i64);
    }
    for c in sub_c.chars() {
        cpu.io_in.push_front(c as u8 as i64);
    }
    cpu.io_in.push_front('n' as u8 as i64);
    cpu.io_in.push_front(10);
}

fn update_view(cpu: &mut Cpu, view: &mut [Vec<char>]) {
    let mut row = 0;
    let mut col = 0;
    while let Some(num) = cpu.io_out.pop_back() {
        if num == 10 {
            row += 1;
            col = 0;
        } else {
            view[row][col] = num as u8 as char;
        }
        if row >= view.len() {
            return;
        }
    }
}

fn run_routine(cpu: &mut Cpu, view: &mut [Vec<char>]) {
    cpu.run();
    update_view(cpu, view);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("no input provided!");
        return;
    }

    let input = get_input(&args[1]);

    let program = get_program(input);
    let mut cpu = Cpu::new();
    cpu.load_program(&program);
    cpu.run();

    let mut view: Vec<Vec<char>> = Vec::new();
    view.push(Vec::new());
    let mut row = 0;

    while let Some(num) = cpu.io_out.pop_back() {
        let c = num as u8 as char;
        // print!("{}", c);
        if c == '\n' {
            view.push(Vec::new());
            row += 1;
        } else {
            view[row].push(c);
        }
    }
    view.pop();
    view.pop();
    print_canvas(&view);
    let alignment = get_alignment_params(&view);
    println!("alignment: {}", alignment);

    cpu.load_program(&program);
    program_robot(&mut cpu);
    run_routine(&mut cpu, &mut view);
    print_canvas(&view);
}
