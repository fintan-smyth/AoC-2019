use core::panic;
use std::{
    collections::{HashMap, VecDeque},
    env, fs,
    hash::Hash,
    io::{Write, stdin, stdout},
    process::{Output, exit},
};

enum Dir {
    North,
    East,
    South,
    West,
}

enum Colour {
    Black,
    White,
}

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
                if self.io_in.is_empty() {
                    self.state = State::Ready;
                    println!("\x1b[35;1mWaiting for IO in...\x1b[m");
                    return;
                }
                let input = self.io_in.pop_back().expect("No io available to read!");
                if let RegMode::Rel = self.reg_mode[0] {
                    self.reg[0] += self.bp;
                }
                self.memory[self.reg[0] as usize] = input;
                println!("\x1b[1;32mINPUT  <\x1b[m {}", input);
            }
            Op::Out => {
                println!("\x1b[1;31mOUTPUT >\x1b[m {}", self.reg[0]);
                self.io_out.push_front(self.reg[0]);
                if let CpuMode::BreakOnOutput = self.mode {
                    self.state = State::Ready;
                }
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

struct Robot {
    cpu: Cpu,
    dir: Dir,
    pos: (i64, i64),
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

fn read_input() -> i64 {
    print!("\x1b[1;32mINPUT  <\x1b[m ");
    stdout().flush().unwrap();

    let mut input = String::new();

    stdin().read_line(&mut input).expect("Failed to read line");

    input.trim().parse().expect("Failed to read input number")
}

fn paint_tile(floor: &mut HashMap<(i64, i64), Colour>, pos: (i64, i64), col: i64) {
    match col {
        0 => floor.insert(pos, Colour::Black),
        1 => floor.insert(pos, Colour::White),
        _ => panic!("Invalid colour provided!"),
    };
}

fn turn_robot(robot: &mut Robot, dir: i64) {
    robot.dir = match dir {
        0 => match robot.dir {
            Dir::North => Dir::West,
            Dir::East => Dir::North,
            Dir::South => Dir::East,
            Dir::West => Dir::South,
        },
        1 => match robot.dir {
            Dir::North => Dir::East,
            Dir::East => Dir::South,
            Dir::South => Dir::West,
            Dir::West => Dir::North,
        },
        _ => panic!("Invalid dir provided"),
    }
}

fn move_robot(robot: &mut Robot) {
    match robot.dir {
        Dir::North => robot.pos.1 -= 1,
        Dir::South => robot.pos.1 += 1,
        Dir::East => robot.pos.0 += 1,
        Dir::West => robot.pos.0 -= 1,
    }
}

fn get_painted(robot: &mut Robot) -> HashMap<(i64, i64), Colour> {
    let mut floor = HashMap::new();

    robot.cpu.io_in.push_front(1);
    floor.insert((0, 0), Colour::White);
    // println!("io_in empty: {}", robot.cpu.io_in.is_empty());
    loop {
        if let State::Halted = robot.cpu.state {
            break;
        }
        robot.cpu.run();
        // println!("First break");
        let colour = robot.cpu.io_out.pop_back().expect("No output from robot!");
        paint_tile(&mut floor, robot.pos, colour);
        let dir = robot.cpu.io_out.pop_back().expect("No output from robot!");
        turn_robot(robot, dir);
        move_robot(robot);
        if let Some(col) = floor.get(&robot.pos) {
            match col {
                Colour::Black => robot.cpu.io_in.push_front(0),
                Colour::White => robot.cpu.io_in.push_front(1),
            }
        } else {
            robot.cpu.io_in.push_front(0);
        }
    }

    floor
}

fn find_boundaries(floor: &HashMap<(i64, i64), Colour>) -> (i64, i64, i64, i64) {
    let mut min_x = i64::MAX;
    let mut min_y = i64::MAX;
    let mut max_x = i64::MIN;
    let mut max_y = i64::MIN;

    for (key, _) in floor {
        let (x, y) = *key;
        if x < min_x {
            min_x = x;
        } else if x > max_x {
            max_x = x;
        }
        if y < min_y {
            min_y = y;
        } else if y > max_y {
            max_y = y;
        }
    }

    (min_x, min_y, max_x, max_y)
}

fn draw_floor(floor: &HashMap<(i64, i64), Colour>) -> Vec<Vec<char>> {
    let (min_x, min_y, max_x, max_y) = find_boundaries(floor);
    let n_rows = max_y - min_y + 1;
    let n_cols = max_x - min_x + 1;
    let mut canvas: Vec<Vec<char>> = Vec::new();
    println!("min: ({},{})", min_x, min_y);
    println!("max: ({},{})", max_x, max_y);

    for _ in 0..n_rows {
        let mut row: Vec<char> = Vec::new();
        for _ in 0..n_cols {
            row.push('.');
        }
        canvas.push(row);
    }

    for (key, val) in floor {
        let (x, y) = ((key.0 - min_x) as usize, (key.1 - min_y) as usize);
        if let Colour::White = val {
            canvas[y][x] = '#';
        }
    }

    canvas
}

fn print_canvas(canvas: &Vec<Vec<char>>) {
    for row in canvas {
        for c in row {
            if *c == '#' {
                print!("\x1b[34m{c}\x1b[m");
            } else {
                print!(" ")
            };
        }
        println!();
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

    let mut robot = Robot {
        cpu: Cpu::new(),
        dir: Dir::North,
        pos: (0, 0),
    };
    robot.cpu.load_program(&program);
    // robot.cpu.mode = CpuMode::BreakOnOutput;

    let floor = get_painted(&mut robot);
    let canvas = draw_floor(&floor);
    print_canvas(&canvas);

    println!("output: {}", floor.len());
}
