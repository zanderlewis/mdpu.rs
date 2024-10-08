use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

struct ProcessingUnit {
    registers: Vec<i32>,
    memory: Vec<i32>,
    stack_pointer: usize,
}

// Define the structure to hold the state after execution
struct ProcessingUnitState {
    registers: Vec<i32>,
    stack: Vec<i32>,
}

// Define opcodes
#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
enum Opcode {
    Nop,
    Add,
    Sub,
    Mul,
    Div,
    Store,
    Load,
    LoadImmediate,
    Push,
    Pop,
    Jmp,
    Jz,
    Jnz,
    Mov,
    Je,
    Jne,
    And,
    Or,
    Xor,
    Not,
    Shl,
    Shr,
    Cmp,
    Test,
    B,
    Bz,
    Bnz,
    Neg,
    Abs,
    Mod,
    Inc,
    Dec,
    Halt,
}

// Define the structure of an instruction
struct Instruction {
    opcode: Opcode,
    reg1: usize,
    reg2: usize,
    reg3: usize,
    addr: usize,
    immediate: i32,
}

impl ProcessingUnit {
    // Function to initialize the processing unit
    fn initialize(num_registers: usize, memory_size: usize) -> Self {
        ProcessingUnit {
            registers: vec![0; num_registers],
            memory: vec![0; memory_size],
            stack_pointer: memory_size - 1, // Initialize stack pointer to the top of the memory
        }
    }

    // Helper function to check register bounds
    fn check_register_bounds(&self, reg: usize) {
        if reg >= self.registers.len() {
            eprintln!("Error: Register index out of bounds: R{}", reg);
            std::process::exit(1);
        }
    }

    // ++++++++++++++++++++++++++++++ Arithmetic operations ++++++++++++++++++++++++++++++ //
    fn add(&mut self, reg1: usize, reg2: usize, reg3: usize) {
        self.check_register_bounds(reg1);
        self.check_register_bounds(reg2);
        self.check_register_bounds(reg3);
        self.registers[reg3] = self.registers[reg1] + self.registers[reg2];
    }

    fn subtract(&mut self, reg1: usize, reg2: usize, reg3: usize) {
        self.check_register_bounds(reg1);
        self.check_register_bounds(reg2);
        self.check_register_bounds(reg3);
        self.registers[reg3] = self.registers[reg1] - self.registers[reg2];
    }

    fn multiply(&mut self, reg1: usize, reg2: usize, reg3: usize) {
        self.check_register_bounds(reg1);
        self.check_register_bounds(reg2);
        self.check_register_bounds(reg3);
        self.registers[reg3] = self.registers[reg1] * self.registers[reg2];
    }

    fn divide(&mut self, reg1: usize, reg2: usize, reg3: usize) {
        self.check_register_bounds(reg1);
        self.check_register_bounds(reg2);
        self.check_register_bounds(reg3);
        if self.registers[reg2] != 0 {
            self.registers[reg3] = self.registers[reg1] / self.registers[reg2];
        } else {
            eprintln!(
                "Error: Division by zero on R{} of value {}",
                reg2, self.registers[reg2]
            );
            std::process::exit(1);
        }
    }

    fn neg(&mut self, reg1: usize, reg2: usize) {
        self.check_register_bounds(reg1);
        self.check_register_bounds(reg2);
        self.registers[reg2] = -self.registers[reg1];
    }

    fn absolute(&mut self, reg1: usize, reg2: usize) {
        self.check_register_bounds(reg1);
        self.check_register_bounds(reg2);
        self.registers[reg2] = self.registers[reg1].abs();
    }

    fn mod_op(&mut self, reg1: usize, reg2: usize, reg3: usize) {
        self.check_register_bounds(reg1);
        self.check_register_bounds(reg2);
        self.check_register_bounds(reg3);
        if self.registers[reg2] != 0 {
            self.registers[reg3] = self.registers[reg1] % self.registers[reg2];
        } else {
            eprintln!(
                "Error: Division by zero on R{} of value {}",
                reg2, self.registers[reg2]
            );
            std::process::exit(1);
        }
    }

    // ++++++++++++++++++++++++++++++ Memory operations ++++++++++++++++++++++++++++++ //
    fn store(&mut self, reg: usize, addr: usize) {
        self.check_register_bounds(reg);
        if addr < self.memory.len() {
            self.memory[addr] = self.registers[reg];
        } else {
            eprintln!("Error: Memory address out of bounds: {}", addr);
            std::process::exit(1);
        }
    }

    fn load(&mut self, addr: usize, reg: usize) {
        self.check_register_bounds(reg);
        if addr < self.memory.len() {
            self.registers[reg] = self.memory[addr];
        } else {
            eprintln!("Error: Memory address out of bounds: {}", addr);
            std::process::exit(1);
        }
    }

    // ++++++++++++++++++++++++++++++ Stack operations ++++++++++++++++++++++++++++++ //
    fn push(&mut self, reg: usize) {
        self.check_register_bounds(reg);
        if self.stack_pointer > 0 {
            self.memory[self.stack_pointer] = self.registers[reg];
            self.stack_pointer -= 1;
        } else {
            eprintln!("Error: Stack overflow on R{}", reg);
            std::process::exit(1);
        }
    }

    fn pop(&mut self, reg: usize) {
        self.check_register_bounds(reg);
        if self.stack_pointer < self.memory.len() - 1 {
            self.stack_pointer += 1;
            self.registers[reg] = self.memory[self.stack_pointer];
        } else {
            eprintln!("Error: Stack underflow on R{}", reg);
            std::process::exit(1);
        }
    }

    fn mov(&mut self, reg1: usize, reg2: usize) {
        self.check_register_bounds(reg1);
        self.check_register_bounds(reg2);
        self.registers[reg1] = self.registers[reg2];
    }
}

// Function to run the program and return the state
fn run(pu: &mut ProcessingUnit, program: &[Instruction], mic: usize) -> ProcessingUnitState {
    execute_program(pu, program, mic);
    // let stack_size = pu.memory.len() - pu.stack_pointer - 1;

    let stack = pu.memory[pu.stack_pointer + 1..].to_vec();
    let registers = pu.registers.clone();

    ProcessingUnitState { registers, stack }
}

// ++++++++++++++++++++++++++++++ Program execution ++++++++++++++++++++++++++++++ //
fn execute_program(pu: &mut ProcessingUnit, program: &[Instruction], mic: usize) {
    let max_instruction_count = mic;
    let mut instruction_count = 0;
    let mut instruction_pointer = 0;

    while instruction_pointer < program.len() {
        if instruction_count >= max_instruction_count {
            eprintln!("Error: Maximum instruction count exceeded, possible infinite loop");
            std::process::exit(1);
        }

        let instr = &program[instruction_pointer];
        match instr.opcode {
            Opcode::Add => pu.add(instr.reg1, instr.reg2, instr.reg3),
            Opcode::Sub => pu.subtract(instr.reg1, instr.reg2, instr.reg3),
            Opcode::Mul => pu.multiply(instr.reg1, instr.reg2, instr.reg3),
            Opcode::Div => pu.divide(instr.reg1, instr.reg2, instr.reg3),
            Opcode::Store => pu.store(instr.reg1, instr.addr),
            Opcode::Load => pu.load(instr.addr, instr.reg1),
            Opcode::LoadImmediate => {
                pu.check_register_bounds(instr.reg1);
                pu.registers[instr.reg1] = instr.immediate;
            }
            Opcode::Push => pu.push(instr.reg1),
            Opcode::Pop => pu.pop(instr.reg1),
            Opcode::Jmp => {
                instruction_pointer = instr.addr;
                continue;
            }
            Opcode::Jz => {
                pu.check_register_bounds(instr.reg1);
                if pu.registers[instr.reg1] == 0 {
                    instruction_pointer = instr.addr;
                    continue;
                }
            }
            Opcode::Jnz => {
                pu.check_register_bounds(instr.reg1);
                if pu.registers[instr.reg1] != 0 {
                    instruction_pointer = instr.addr;
                    continue;
                }
            }
            Opcode::Mov => pu.mov(instr.reg1, instr.reg2),
            Opcode::Je => {
                pu.check_register_bounds(instr.reg1);
                pu.check_register_bounds(instr.reg2);
                if pu.registers[instr.reg1] == pu.registers[instr.reg2] {
                    instruction_pointer = instr.addr;
                }
            }
            Opcode::Jne => {
                pu.check_register_bounds(instr.reg1);
                pu.check_register_bounds(instr.reg2);
                if pu.registers[instr.reg1] != pu.registers[instr.reg2] {
                    instruction_pointer = instr.addr;
                }
            }
            Opcode::And => {
                pu.check_register_bounds(instr.reg1);
                pu.check_register_bounds(instr.reg2);
                pu.check_register_bounds(instr.reg3);
                pu.registers[instr.reg3] = pu.registers[instr.reg1] & pu.registers[instr.reg2];
            }
            Opcode::Or => {
                pu.check_register_bounds(instr.reg1);
                pu.check_register_bounds(instr.reg2);
                pu.check_register_bounds(instr.reg3);
                pu.registers[instr.reg3] = pu.registers[instr.reg1] | pu.registers[instr.reg2];
            }
            Opcode::Xor => {
                pu.check_register_bounds(instr.reg1);
                pu.check_register_bounds(instr.reg2);
                pu.check_register_bounds(instr.reg3);
                pu.registers[instr.reg3] = pu.registers[instr.reg1] ^ pu.registers[instr.reg2];
            }
            Opcode::Not => {
                pu.check_register_bounds(instr.reg1);
                pu.check_register_bounds(instr.reg2);
                pu.registers[instr.reg2] = !pu.registers[instr.reg1];
            }
            Opcode::Shl => {
                pu.check_register_bounds(instr.reg1);
                pu.check_register_bounds(instr.reg2);
                pu.check_register_bounds(instr.reg3);
                pu.registers[instr.reg3] = pu.registers[instr.reg1] << pu.registers[instr.reg2];
            }
            Opcode::Shr => {
                pu.check_register_bounds(instr.reg1);
                pu.check_register_bounds(instr.reg2);
                pu.check_register_bounds(instr.reg3);
                pu.registers[instr.reg3] = pu.registers[instr.reg1] >> pu.registers[instr.reg2];
            }
            Opcode::Cmp => {
                pu.check_register_bounds(instr.reg1);
                pu.check_register_bounds(instr.reg2);
                pu.check_register_bounds(instr.reg3);
                pu.registers[instr.reg3] = pu.registers[instr.reg1] - pu.registers[instr.reg2];
            }
            Opcode::Test => {
                pu.check_register_bounds(instr.reg1);
                pu.check_register_bounds(instr.reg2);
                pu.check_register_bounds(instr.reg3);
                pu.registers[instr.reg3] = pu.registers[instr.reg1] & pu.registers[instr.reg2];
            }
            Opcode::B => {
                instruction_pointer = instr.addr;
                continue;
            }
            Opcode::Bz => {
                pu.check_register_bounds(instr.reg1);
                if pu.registers[instr.reg1] == 0 {
                    instruction_pointer = instr.addr;
                    continue;
                }
            }
            Opcode::Bnz => {
                pu.check_register_bounds(instr.reg1);
                if pu.registers[instr.reg1] != 0 {
                    instruction_pointer = instr.addr;
                    continue;
                }
            }
            Opcode::Neg => pu.neg(instr.reg1, instr.reg2),
            Opcode::Abs => pu.absolute(instr.reg1, instr.reg2),
            Opcode::Mod => pu.mod_op(instr.reg1, instr.reg2, instr.reg3),
            Opcode::Inc => {
                pu.check_register_bounds(instr.reg1);
                pu.registers[instr.reg1] += 1;
            }
            Opcode::Dec => {
                pu.check_register_bounds(instr.reg1);
                pu.registers[instr.reg1] -= 1;
            }
            Opcode::Nop => {}
            Opcode::Halt => break, // Stop execution
        }

        instruction_count += 1;
        instruction_pointer += 1;
    }
}

// Function to parse the dimensions
fn parse_dimensions(dimensions: &str) -> usize {
    let dims: Vec<usize> = dimensions
        .split('x')
        .map(|dim| {
            dim.parse::<usize>()
                .expect("Error: Invalid dimension, must be a positive integer")
        })
        .collect();
    dims.iter().product()
}

// Modify the main function to load instructions from a file
fn main() {
    use std::env;

    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!(
            "Usage: {} <register_size_dimensions> <memory_size_dimensions> <program_file>",
            args[0]
        );
        std::process::exit(1);
    }

    // Parse the dimensions for registers and memory
    let total_registers = parse_dimensions(&args[1]);
    let total_memory = parse_dimensions(&args[2]);
    let program_file = &args[3];

    let mut pu = ProcessingUnit::initialize(total_registers, total_memory);

    // Load the program from a file
    let program = load_program(program_file).expect("Failed to load program");

    let mic = 1000; // Maximum instruction count
    let state = run(&mut pu, &program, mic);

    println!("Registers: {:?}", state.registers);
    println!("Stack: {:?}", state.stack);
}

// Function to load a program from a file
fn load_program(filename: &str) -> Result<Vec<Instruction>, io::Error> {
    let path = Path::new(filename);
    let file = File::open(&path)?;
    let lines = io::BufReader::new(file).lines();

    let mut program = Vec::new();

    for line in lines {
        if let Ok(instr_str) = line {
            if let Some(instr) = parse_instruction(&instr_str) {
                program.push(instr);
            }
        }
    }

    Ok(program)
}

// Function to parse an instruction from a line of text
fn parse_instruction(line: &str) -> Option<Instruction> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    // Check for comment or empty line
    if parts.is_empty() || parts[0].starts_with("//") || parts[0].starts_with("NOP\n") {
        return Some(Instruction {
            opcode: Opcode::Nop,
            reg1: 0,
            reg2: 0,
            reg3: 0,
            addr: 0,
            immediate: 0,
        });
    }

    let opcode = match parts[0] {
        "ADD" => Opcode::Add,
        "SUB" => Opcode::Sub,
        "MUL" => Opcode::Mul,
        "DIV" => Opcode::Div,
        "STORE" => Opcode::Store,
        "LOAD" => Opcode::Load,
        "LI" => Opcode::LoadImmediate,
        "PUSH" => Opcode::Push,
        "POP" => Opcode::Pop,
        "JMP" => Opcode::Jmp,
        "JZ" => Opcode::Jz,
        "JNZ" => Opcode::Jnz,
        "MOV" => Opcode::Mov,
        "JE" => Opcode::Je,
        "JNE" => Opcode::Jne,
        "AND" => Opcode::And,
        "OR" => Opcode::Or,
        "XOR" => Opcode::Xor,
        "NOT" => Opcode::Not,
        "SHL" => Opcode::Shl,
        "SHR" => Opcode::Shr,
        "CMP" => Opcode::Cmp,
        "TEST" => Opcode::Test,
        "B" => Opcode::B,
        "BZ" => Opcode::Bz,
        "BNZ" => Opcode::Bnz,
        "NEG" => Opcode::Neg,
        "ABS" => Opcode::Abs,
        "MOD" => Opcode::Mod,
        "INC" => Opcode::Inc,
        "DEC" => Opcode::Dec,
        "HALT" => Opcode::Halt,
        _ => {
            eprintln!("Unknown opcode: {}", parts[0]);
            return None;
        }
    };

    let reg1 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let reg2 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    let reg3 = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
    let addr = parts.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
    let immediate = parts.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);

    Some(Instruction {
        opcode,
        reg1,
        reg2,
        reg3,
        addr,
        immediate,
    })
}
