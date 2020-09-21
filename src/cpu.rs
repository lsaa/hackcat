use crate::memory::{ROM32K, RAM16K};

pub struct CPU {
    pub register_a: u16,
    pub register_d: u16,
    pub pc: u16,
    pub cc: u128,
    pub rom: ROM32K,
    pub ram: RAM16K,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            register_a: 0,
            register_d: 0,
            pc: 0,
            cc: 0,
            ram: RAM16K::new(),
            rom: ROM32K::new()
        }
    }

    pub fn reset(&mut self) {
        let rom = &self.rom.raw_program;
        let mut new_rom = ROM32K::new();
        new_rom.load_program(rom.clone());
        *self = Self {
            register_a: 0,
            register_d: 0,
            pc: 0,
            cc: 0,
            ram: RAM16K::new(),
            rom: new_rom
        }
    }

    pub fn execute(&mut self) {
        let hi = *self.rom.read_byte(self.pc) as u16;
        let lo = *self.rom.read_byte(self.pc + 1) as u16;
        let instruction: u16 = (hi << 8) | lo;
        self.cc += 1;

        match instruction & 0b1000000000000000 {
            0b0000000000000000 => self.a_instruction(instruction),
            0b1000000000000000 => self.c_instruction(instruction),
            _ => ()
        }
    }

    pub fn a_instruction(&mut self, opcode: u16) {
        self.register_a = opcode & 0b0111111111111111;
        self.pc += 2;
    }

    pub fn c_instruction(&mut self, opcode: u16) {
        let out: u16 = match opcode & 0b0001000000000000 {
            0b000000000 => {
                match (opcode & 0b0000111111000000) >> 6 {
                    0b101010 => 0,
                    0b111111 => 1,
                    0b111010 => -1_i16 as u16,
                    0b001100 => self.register_d,
                    0b110000 => self.register_a,
                    0b001101 => !self.register_d,
                    0b110001 => !self.register_a,
                    0b001111 => (!self.register_d).overflowing_add(1).0,
                    0b110011 => (!self.register_a).overflowing_add(1).0,
                    0b011111 => self.register_d.overflowing_add(1).0,
                    0b110111 => self.register_a.overflowing_add(1).0,
                    0b001110 => self.register_d.overflowing_add((!1_u16).overflowing_add(1).0).0,
                    0b110010 => self.register_a.overflowing_add((!1_u16).overflowing_add(1).0).0,
                    0b000010 => self.register_d.overflowing_add(self.register_a).0,
                    0b010011 => self.register_d.overflowing_add((!self.register_a).overflowing_add(1).0).0,
                    0b000111 => self.register_a.overflowing_add((!self.register_d).overflowing_add(1).0).0,
                    0b000000 => self.register_d & self.register_a,
                    0b010101 => self.register_d | self.register_a,
                    _ => 0
                }
            }, 
            0b0001000000000000 => {
                match (opcode & 0b0000111111000000) >> 6 {
                    0b110000 => *self.ram.read_u16(self.register_a),
                    0b110001 => !*self.ram.read_u16(self.register_a),
                    0b110011 => (!*self.ram.read_u16(self.register_a)).overflowing_add(1).0,
                    0b110111 => (*self.ram.read_u16(self.register_a)).overflowing_add(1).0,
                    0b110010 => (*self.ram.read_u16(self.register_a)).overflowing_add((!1_u16).overflowing_add(1).0).0,
                    0b000010 => self.register_d.overflowing_add(*self.ram.read_u16(self.register_a)).0 ,
                    0b010011 => self.register_d.overflowing_add((!*self.ram.read_u16(self.register_a)).overflowing_add(1).0).0,
                    0b000111 => (*self.ram.read_u16(self.register_a)).overflowing_add((!self.register_d).overflowing_add(1).0).0,
                    0b000000 => self.register_d & *self.ram.read_u16(self.register_a),
                    0b010101 => *self.ram.read_u16(self.register_a) | self.register_d,
                    _ => 0
                }
            },
            _ => 0
        };

        let v = (opcode & 0b111000) >> 3;
        let (a, d , m) = (v & 0b100 == 0b100, v & 0b010 == 0b010, v & 0b001 == 0b001);

        if m {self.ram.write_u16(self.register_a, out);}
        if a {self.register_a = out}
        if d {self.register_d = out}

        let out = out as i16;
        match opcode & 0b111 {
            0b000 => self.pc += 2,
            0b001 => if out > 0 {self.pc = self.register_a*2} else { self.pc += 2 },
            0b010 => if out == 0 {self.pc = self.register_a*2} else { self.pc += 2 },
            0b011 => if out >= 0 {self.pc = self.register_a*2} else { self.pc += 2 },
            0b100 => if out < 0 {self.pc = self.register_a*2} else { self.pc += 2 },
            0b101 => if out != 0 {self.pc = self.register_a*2} else { self.pc += 2 },
            0b110 => if out <= 0 {self.pc = self.register_a*2} else { self.pc += 2 },
            0b111 => self.pc = self.register_a*2,
            _ => ()
        }

    }

    pub fn disassemble_loaded_rom(&self) -> Vec<String> {
        let mut instructions = vec![];
        for (pc, _instruction) in self.rom.raw_program.iter().enumerate().step_by(2) {
            if pc + 1 >= self.rom.raw_program.len() { break; }
            let hi = *self.rom.read_byte(pc as u16) as u16;
            let lo = *self.rom.read_byte((pc + 1) as u16) as u16;
            let instruction: u16 = (hi << 8) | lo;    
            match instruction & 0b1000000000000000 {
                0b0000000000000000 => instructions.push(format!("{}: @{}",pc/2, instruction & 0b0111111111111111)),
                0b1000000000000000 => {
                    let (dst, cmp, jmp) = CPU::disassemble_c(instruction);
                    instructions.push(format!("{}: {}{}; {}", pc/2, dst, cmp, jmp))
                },
                _ => ()
            }
        }
        return instructions;
    } 

    fn disassemble_c(opcode: u16) -> (String, String, String) {
        let math: &str = match opcode & 0b0001000000000000 {
            0b000000000 => {
                match (opcode & 0b0000111111000000) >> 6 {
                    0b101010 => "0",
                    0b111111 => "1",
                    0b111010 => "-1",
                    0b001100 => "D",
                    0b110000 => "A",
                    0b001101 => "!D",
                    0b110001 => "!A",
                    0b001111 => "-D",
                    0b110011 => "-A",
                    0b011111 => "D+1",
                    0b110111 => "A+1",
                    0b001110 => "D-1",
                    0b110010 => "A-1",
                    0b000010 => "D+A",
                    0b010011 => "D-A",
                    0b000111 => "A-D",
                    0b000000 => "D&A",
                    0b010101 => "D|A",
                    _ => ""
                }
            }, 
            0b0001000000000000 => {
                match (opcode & 0b0000111111000000) >> 6 {
                    0b110000 => "M",
                    0b110001 => "!M",
                    0b110011 => "-M",
                    0b110111 => "M+1",
                    0b110010 => "M-1",
                    0b000010 => "D+M",
                    0b010011 => "D-M",
                    0b000111 => "M-D",
                    0b000000 => "D&M",
                    0b010101 => "M|A",
                    _ => ""
                }
            },
            _ => ""
        };

        let dst = match (opcode & 0b111000) >> 3 {
            0b000 => "",
            0b001 => "M = ",
            0b010 => "D = ",
            0b011 => "DM = ",
            0b100 => "A = ",
            0b101 => "AM = ",
            0b110 => "AD = ",
            0b111 => "ADM = ",
            _ => ""
        };

        let jmp = match opcode & 0b111 {
            0b000 => "",
            0b001 => "JGT",
            0b010 => "JEQ",
            0b011 => "JGE",
            0b100 => "JLT",
            0b101 => "JNE",
            0b110 => "JLE",
            0b111 => "JMP",
            _ => ""
        };

        return (String::from(dst), String::from(math), String::from(jmp));
    }
}