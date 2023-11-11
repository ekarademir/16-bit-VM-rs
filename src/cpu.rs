use crate::memory::Memory;
use std::fmt::Debug;

pub struct Cpu {
    memory: Memory,
    register: Memory,
}

impl Cpu {
    pub fn new(memory: Memory) -> Cpu {
        let register_names = [
            Register::InstructionPointer,
            Register::Accumulator,
            Register::Register1,
            Register::Register2,
            Register::Register3,
            Register::Register4,
            Register::Register5,
            Register::Register6,
            Register::Register7,
            Register::Register8,
        ];

        let register = Memory::new(register_names.len() * 2);

        Cpu { memory, register }
    }

    pub fn step(&mut self) {
        let instruction = self.fetch();
        self.execute(instruction.into());
    }

    #[allow(dead_code)]
    pub fn peek_register(&self, name: Register) -> u16 {
        self.get_register(name)
    }

    pub fn step_n(&mut self, n: usize) {
        for _ in 0..n {
            self.step();
        }
    }
}

impl Cpu {
    fn register_map(&self, name: Register) -> usize {
        name as usize * 2
    }

    fn get_register(&self, name: Register) -> u16 {
        self.register.get_word(self.register_map(name))
    }

    fn set_register(&mut self, name: Register, value: u16) {
        self.register.set_word(self.register_map(name), value);
    }

    fn fetch(&mut self) -> u8 {
        let next_instruction_addr = self.get_register(Register::InstructionPointer);
        self.set_register(Register::InstructionPointer, next_instruction_addr + 1);

        self.memory.get_byte(next_instruction_addr as usize)
    }

    fn fetch16(&mut self) -> u16 {
        let next_instruction_addr = self.get_register(Register::InstructionPointer);
        self.set_register(Register::InstructionPointer, next_instruction_addr + 2);
        self.memory.get_word(next_instruction_addr as usize)
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::MovLitR1 => {
                let value = self.fetch16();
                self.set_register(Register::Register1, value);
            }
            Instruction::MovLitR2 => {
                let value = self.fetch16();
                self.set_register(Register::Register2, value);
            }
            Instruction::AddRegReg => {
                let register1 = self.fetch();
                let register2 = self.fetch();

                let value1 = self.register.get_word(register1 as usize * 2);
                let value2 = self.register.get_word(register2 as usize * 2);

                self.set_register(Register::Accumulator, value1 + value2);
            }
            _ => {}
        }
    }
}

impl Debug for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CPU: {:?}", self.register)
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Register {
    InstructionPointer,
    Accumulator,
    Register1,
    Register2,
    Register3,
    Register4,
    Register5,
    Register6,
    Register7,
    Register8,
}

#[repr(u8)]
pub enum Instruction {
    Noop = 0x00,
    MovLitR1 = 0x10,
    MovLitR2 = 0x11,
    AddRegReg = 0x12,
}

impl From<u8> for Instruction {
    fn from(value: u8) -> Self {
        match value {
            0x10 => Instruction::MovLitR1,
            0x11 => Instruction::MovLitR2,
            0x12 => Instruction::AddRegReg,
            _ => Instruction::Noop,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Cpu, Instruction, Register};
    use crate::memory::Memory;

    #[test]
    fn bumps_instruction_pointer_at_every_step() {
        let memory = Memory::new(256);
        let mut cpu = Cpu::new(memory);

        assert_eq!(cpu.peek_register(Register::InstructionPointer), 0);
        cpu.step();
        assert_eq!(cpu.peek_register(Register::InstructionPointer), 1);
        cpu.step();
        assert_eq!(cpu.peek_register(Register::InstructionPointer), 2);
    }

    #[test]
    fn test_move_to_r1() {
        let mut memory = Memory::new(32);

        // mov 0x1234, r1
        let mut i = 0;
        memory.set_byte(i, Instruction::MovLitR1 as u8);
        i += 1;
        memory.set_byte(i, 0x12);
        i += 1;
        memory.set_byte(i, 0x34);

        let mut cpu = Cpu::new(memory);
        cpu.step();

        assert_eq!(cpu.peek_register(Register::Register1), 0x1234);
    }

    #[test]
    fn test_move_to_r2() {
        let mut memory = Memory::new(32);

        // mov 0x1234, r2
        let mut i = 0;
        memory.set_byte(i, Instruction::MovLitR2 as u8);
        i += 1;
        memory.set_byte(i, 0x12);
        i += 1;
        memory.set_byte(i, 0x34);

        let mut cpu = Cpu::new(memory);
        cpu.step();

        assert_eq!(cpu.peek_register(Register::Register1), 0x0000);
        assert_eq!(cpu.peek_register(Register::Register2), 0x1234);
    }

    #[test]
    fn adds_two_numbers() {
        let mut memory = Memory::new(32);

        // mov 0x1234, r1
        let mut i = 0;
        memory.set_byte(i, Instruction::MovLitR1 as u8);
        i += 1;
        memory.set_byte(i, 0x12);
        i += 1;
        memory.set_byte(i, 0x34);
        i += 1;

        // mov 0xabcd, r2
        memory.set_byte(i, Instruction::MovLitR2 as u8);
        i += 1;
        memory.set_byte(i, 0xab);
        i += 1;
        memory.set_byte(i, 0xcd);
        i += 1;

        // add r1, r2
        memory.set_byte(i, Instruction::AddRegReg as u8);
        i += 1;
        memory.set_byte(i, Register::Register1 as u8);
        i += 1;
        memory.set_byte(i, Register::Register2 as u8);

        let mut cpu = Cpu::new(memory);
        cpu.step_n(3);

        assert_eq!(cpu.peek_register(Register::Register1), 0x1234);
        assert_eq!(cpu.peek_register(Register::Register2), 0xabcd);
        assert_eq!(cpu.peek_register(Register::Accumulator), 0xbe01);
    }
}
