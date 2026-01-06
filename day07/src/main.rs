use std::{
    collections::VecDeque,
    env, fs,
    io::{Write, stdin, stdout},
    process::Output,
};

#[derive(PartialEq)]
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
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
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
        Op::Add => cpu.memory[cpu.reg[2] as usize] = cpu.reg[0] + cpu.reg[1],
        Op::Mul => cpu.memory[cpu.reg[2] as usize] = cpu.reg[0] * cpu.reg[1],
        Op::In => {
            let input = cpu.io_in.pop_back().expect("No io available to read!");
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
            if cpu.reg[0] < cpu.reg[1] {
                cpu.memory[cpu.reg[2] as usize] = 1;
            } else {
                cpu.memory[cpu.reg[2] as usize] = 0;
            }
        }
        Op::Cmp => {
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

fn load_program(cpu: &mut Cpu, program: &[i64]) {
    cpu.ip = 0;
    cpu.io_in.clear();
    cpu.io_out.clear();
    cpu.state = State::Ready;
    cpu.memory.fill(0);
    cpu.memory[0..program.len()].copy_from_slice(program);
}

fn run_cpu(cpu: &mut Cpu) {
    cpu.state = State::Active;
    loop {
        // print_prog(&memory, cpu.ip);
        let instruction = cpu.memory[cpu.ip];
        let cmd: Cmd = get_cmd(cpu.memory[cpu.ip]).expect("Invalid opcode encountered!");
        get_mode(&mut cpu.reg_mode, instruction, cmd.n_operands);

        for i in 0..cmd.n_operands {
            cpu.reg[i] = cpu.memory[cpu.ip + i + 1];
            // println!("{}", cpu.reg[i]);
        }

        cpu.ip += cmd.n_operands + 1;
        execute_cmd(cpu, cmd);

        let State::Active = cpu.state else {
            break;
        };
    }
}

fn execute_program(cpu: &mut Cpu, program: &[i64]) {
    load_program(cpu, program);
    run_cpu(cpu);
}

fn get_max_output(program: &[i64]) -> i64 {
    let mut max_output = i64::MIN;
    let mut phases: [i64; 5] = [-1; 5];
    let mut max_phases: [i64; 5] = [0; 5];

    let mut amps: [Cpu; 5] = std::array::from_fn(|_| Cpu::new());

    println!("-----------------------");
    for phase_a in 0..5 {
        phases[0] = phase_a;
        for phase_b in 0..5 {
            if phases.contains(&phase_b) {
                continue;
            }
            phases[1] = phase_b;
            for phase_c in 0..5 {
                if phases.contains(&phase_c) {
                    continue;
                }
                phases[2] = phase_c;
                for phase_d in 0..5 {
                    if phases.contains(&phase_d) {
                        continue;
                    }
                    phases[3] = phase_d;
                    for phase_e in 0..5 {
                        if phases.contains(&phase_e) {
                            continue;
                        }
                        phases[4] = phase_e;

                        println!("\x1b[35m{:?}\x1b[m", phases);
                        load_program(&mut amps[0], program);
                        amps[0].io_in.push_front(phases[0]);
                        amps[0].io_in.push_front(0);
                        run_cpu(&mut amps[0]);
                        for i in 1..phases.len() {
                            load_program(&mut amps[i], program);
                            amps[i].io_in.push_front(phases[i]);
                            amps[i].io_in.push_front(
                                amps[i - 1].io_out.pop_back().expect("No io out from cpu"),
                            );
                            run_cpu(&mut amps[i]);
                        }

                        let output = amps[4]
                            .io_out
                            .pop_back()
                            .expect("No final output from program.");
                        if output > max_output {
                            max_output = output;
                            max_phases = phases;
                        }
                    }
                    phases[4] = -1;
                }
                phases[3] = -1;
            }
            phases[2] = -1;
        }
        phases[1] = -1;
    }

    println!("\x1b[34m{:?}\x1b[m", max_phases);
    max_output
}

fn run_feedback_loop(amps: &mut [Cpu], output: &mut i64) {
    amps[4].io_out.push_front(0);
    while let State::Ready = amps[4].state {
        println!("\x1b[34m### Amp A ###\x1b[m");

        let Some(input) = amps[4].io_out.pop_back() else {
            println!("\x1b[1;31mNo input available: exiting loop...");
            return;
        };
        amps[0].io_in.push_front(input);
        run_cpu(&mut amps[0]);

        for i in 1..amps.len() {
            println!(
                "\x1b[34m### Amp {} ###\x1b[m",
                ('A' as u8 + i as u8) as char
            );

            let Some(input) = amps[i - 1].io_out.pop_back() else {
                println!("\x1b[1;31mNo input available: exiting loop...");
                return;
            };
            amps[i].io_in.push_front(input);
            run_cpu(&mut amps[i]);
        }
        *output = *amps[4]
            .io_out
            .back()
            .expect("No final output from program.");
    }
}

// fn get_max_feedback_phase(amps: &mut [Cpu], phases: &[i64], )

fn get_max_feedback(program: &[i64]) -> i64 {
    let mut max_output = i64::MIN;
    let mut phases: [i64; 5] = [-1; 5];
    let mut max_phases: [i64; 5] = [0; 5];
    let mut output = 0;

    let mut amps: [Cpu; 5] = std::array::from_fn(|_| Cpu::new());
    for amp in &mut amps {
        amp.mode = CpuMode::BreakOnOutput;
    }

    println!("-----------------------");
    for phase_a in 5..10 {
        phases[0] = phase_a;
        for phase_b in 5..10 {
            if phases.contains(&phase_b) {
                continue;
            }
            phases[1] = phase_b;
            for phase_c in 5..10 {
                if phases.contains(&phase_c) {
                    continue;
                }
                phases[2] = phase_c;
                for phase_d in 5..10 {
                    if phases.contains(&phase_d) {
                        continue;
                    }
                    phases[3] = phase_d;
                    for phase_e in 5..10 {
                        if phases.contains(&phase_e) {
                            continue;
                        }
                        phases[4] = phase_e;

                        println!("\x1b[35m{:?}\x1b[m", phases);
                        for i in 0..5 {
                            load_program(&mut amps[i], program);
                            amps[i].io_in.push_front(phases[i]);
                        }

                        run_feedback_loop(&mut amps, &mut output);

                        if output > max_output {
                            max_output = output;
                            max_phases = phases;
                        }
                    }
                    phases[4] = -1;
                }
                phases[3] = -1;
            }
            phases[2] = -1;
        }
        phases[1] = -1;
    }

    println!("\x1b[34m{:?}\x1b[m", max_phases);
    max_output
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

    // let output = get_max_output(&program);
    let output = get_max_feedback(&program);

    println!("output: {output}");
}
