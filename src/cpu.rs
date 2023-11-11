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

    pub fn peek_tape(&self, address: usize) -> Vec<u8> {
        self.memory.peek(address)
    }

    pub fn peek(&self, address: usize) -> u16 {
        self.memory.get_word(address)
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
            Instruction::MovLitReg => {
                let value = self.fetch16();
                let register = self.fetch();
                self.set_register(register.into(), value);
            }
            Instruction::MovRegReg => {
                let register_from = self.fetch();
                let register_to = self.fetch();
                let value = self.get_register(register_from.into());
                self.set_register(register_to.into(), value);
            }
            Instruction::MovMemReg => {
                let address = self.fetch16();
                let register_to = self.fetch();
                let value = self.memory.get_word(address as usize);
                self.set_register(register_to.into(), value);
            }
            Instruction::MovRegMem => {
                let register_from = self.fetch();
                let address = self.fetch16();
                let value = self.get_register(register_from.into());
                self.memory.set_word(address as usize, value);
            }
            Instruction::AddRegReg => {
                let register1 = self.fetch();
                let register2 = self.fetch();

                let value1 = self.register.get_word(register1 as usize * 2);
                let value2 = self.register.get_word(register2 as usize * 2);

                self.set_register(Register::Accumulator, value1 + value2);
            }
            Instruction::JmpNotEq => {
                let value = self.fetch16();
                let address = self.fetch16();

                let acc_value = self.get_register(Register::Accumulator);

                if value != acc_value {
                    self.set_register(Register::InstructionPointer, address);
                }
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
    None,
}

impl From<u8> for Register {
    fn from(value: u8) -> Self {
        match value {
            0 => Register::InstructionPointer,
            1 => Register::Accumulator,
            2 => Register::Register1,
            3 => Register::Register2,
            4 => Register::Register3,
            5 => Register::Register4,
            6 => Register::Register5,
            7 => Register::Register6,
            8 => Register::Register7,
            9 => Register::Register8,
            _ => Register::None,
        }
    }
}

#[repr(u8)]
pub enum Instruction {
    Noop = 0x00,
    MovLitReg = 0x10,
    MovRegReg = 0x11,
    MovRegMem = 0x12,
    MovMemReg = 0x13,
    AddRegReg = 0x14,
    JmpNotEq = 0x15,
}

impl From<u8> for Instruction {
    fn from(value: u8) -> Self {
        match value {
            0x10 => Instruction::MovLitReg,
            0x11 => Instruction::MovRegReg,
            0x12 => Instruction::MovRegMem,
            0x13 => Instruction::MovMemReg,
            0x14 => Instruction::AddRegReg,
            0x15 => Instruction::JmpNotEq,
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
    fn test_move_lit_to_reg() {
        let mut memory = Memory::new(32);

        // mov 0x1234, r1
        let mut i = 0;
        memory.set_byte(i, Instruction::MovLitReg as u8);
        i += 1;
        memory.set_byte(i, 0x12);
        i += 1;
        memory.set_byte(i, 0x34);
        i += 1;
        memory.set_byte(i, Register::Register1 as u8);

        let mut cpu = Cpu::new(memory);
        cpu.step();

        assert_eq!(cpu.peek_register(Register::Register1), 0x1234);
    }

    #[test]
    fn test_move_mem_to_reg() {
        let mut memory = Memory::new(256 * 256);

        // mov #1000, r1
        let mut i = 0;
        memory.set_byte(i, Instruction::MovMemReg as u8);
        i += 1;
        memory.set_byte(i, 0x10);
        i += 1;
        memory.set_byte(i, 0x00);
        i += 1;
        memory.set_byte(i, Register::Register2 as u8);

        memory.set_byte(0x1000, 0x42);
        memory.set_byte(0x1001, 0x43);

        let mut cpu = Cpu::new(memory);
        cpu.step();

        assert_eq!(cpu.peek_register(Register::Register2), 0x4243);
    }

    #[test]
    fn test_move_reg_to_mem() {
        let mut memory = Memory::new(256 * 256);

        // mov 0x1234, r1
        let mut i = 0;
        memory.set_byte(i, Instruction::MovLitReg as u8);
        i += 1;
        memory.set_byte(i, 0x12);
        i += 1;
        memory.set_byte(i, 0x34);
        i += 1;
        memory.set_byte(i, Register::Register3 as u8);

        // mov r1, #1000
        i += 1;
        memory.set_byte(i, Instruction::MovRegMem as u8);
        i += 1;
        memory.set_byte(i, Register::Register3 as u8);
        i += 1;
        memory.set_byte(i, 0x10);
        i += 1;
        memory.set_byte(i, 0x00);

        let mut cpu = Cpu::new(memory);
        cpu.step_n(2);

        assert_eq!(cpu.peek_tape(0x1000), [0x12, 0x34, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn adds_two_numbers() {
        let mut memory = Memory::new(32);

        // mov 0x1234, r1
        let mut i = 0;
        memory.set_byte(i, Instruction::MovLitReg as u8);
        i += 1;
        memory.set_byte(i, 0x12);
        i += 1;
        memory.set_byte(i, 0x34);
        i += 1;
        memory.set_byte(i, Register::Register1 as u8);

        // mov 0xabcd, r2
        i += 1;
        memory.set_byte(i, Instruction::MovLitReg as u8);
        i += 1;
        memory.set_byte(i, 0xab);
        i += 1;
        memory.set_byte(i, 0xcd);
        i += 1;
        memory.set_byte(i, Register::Register2 as u8);

        // add r1, r2
        i += 1;
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

    #[test]
    fn counts_to_three() {
        let mut memory = Memory::new(256 * 256);

        // start:
        //   mov #0x0100, r1
        //   mov 0x0001, r2
        //   add r1, r2
        //   mov acc, #0100
        //   jne 0x0003, start:

        // mov #0x0100, r1
        let mut i = 0;
        memory.set_byte(i, Instruction::MovMemReg as u8);
        i += 1;
        memory.set_byte(i, 0x01);
        i += 1;
        memory.set_byte(i, 0x00);
        i += 1;
        memory.set_byte(i, Register::Register1 as u8);

        // mov 0x0001, r2
        i += 1;
        memory.set_byte(i, Instruction::MovLitReg as u8);
        i += 1;
        memory.set_byte(i, 0x00);
        i += 1;
        memory.set_byte(i, 0x01);
        i += 1;
        memory.set_byte(i, Register::Register2 as u8);

        // add r1, r2
        i += 1;
        memory.set_byte(i, Instruction::AddRegReg as u8);
        i += 1;
        memory.set_byte(i, Register::Register1 as u8);
        i += 1;
        memory.set_byte(i, Register::Register2 as u8);

        // mov acc, #0100
        i += 1;
        memory.set_byte(i, Instruction::MovRegMem as u8);
        i += 1;
        memory.set_byte(i, Register::Accumulator as u8);
        i += 1;
        memory.set_byte(i, 0x01);
        i += 1;
        memory.set_byte(i, 0x00);

        // jne 0x0003, start:
        i += 1;
        memory.set_byte(i, Instruction::JmpNotEq as u8);
        i += 1;
        memory.set_byte(i, 0x00);
        i += 1;
        memory.set_byte(i, 0x03);
        i += 1;
        memory.set_byte(i, 0x00);
        i += 1;
        memory.set_byte(i, 0x00);

        let mut cpu = Cpu::new(memory);
        cpu.step_n(15);

        assert_eq!(cpu.peek_register(Register::Accumulator), 0x0003);
        assert_eq!(cpu.peek(0x0100), 0x0003);
    }
}
