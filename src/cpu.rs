use crate::memory::Memory;
use std::fmt::Debug;

pub struct Cpu {
    memory: Memory,
    register: Memory,
    register_names: [Register; 12],
    stack_frame_size: usize,
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
            Register::StackPointer,
            Register::FramePointer,
        ];

        let mut register = Memory::new(register_names.len() * 2);

        let bottom_of_stack = memory.byte_length() - 1 - 1;
        let stack_pointer_pointer = Register::StackPointer as usize * 2;
        let frame_pointer_pointer = Register::FramePointer as usize * 2;
        register.set_word(stack_pointer_pointer, bottom_of_stack as u16);
        register.set_word(frame_pointer_pointer, bottom_of_stack as u16);

        Cpu {
            memory,
            register,
            register_names,
            stack_frame_size: 0,
        }
    }

    pub fn step(&mut self) {
        let instruction = self.fetch();
        self.execute(instruction.into());
    }

    pub fn peek_tape(&self, address: usize) -> Vec<u8> {
        self.memory.peek(address, 8)
    }

    pub fn peek_stack(&self) -> Vec<u8> {
        let start = self.get_register(Register::StackPointer) as usize;
        let end = self.memory.byte_length();
        self.memory.peek(start, end)
    }

    pub fn peek(&self, address: usize) -> u16 {
        self.memory.get_word(address)
    }

    pub fn peek_register(&self, register: Register) -> u16 {
        self.get_register(register)
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
        self.get_register_at(self.register_map(name))
    }

    fn get_register_at(&self, index: usize) -> u16 {
        self.register.get_word(index)
    }

    fn set_register(&mut self, name: Register, value: u16) {
        self.set_register_at(self.register_map(name), value);
    }

    fn set_register_at(&mut self, index: usize, value: u16) {
        self.register.set_word(index, value);
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

    fn fetch_register_index(&mut self) -> usize {
        let address = self.fetch();
        address as usize % self.register_names.len()
    }

    fn push(&mut self, value: u16) {
        let stack_pointer = self.get_register(Register::StackPointer);
        self.memory.set_word(stack_pointer as usize, value);
        // stack grows up, 2 bytes at a time
        self.set_register(Register::StackPointer, stack_pointer - 2);
        self.stack_frame_size += 2;
    }

    fn pop(&mut self) -> u16 {
        let next_stack_pointer = self.get_register(Register::StackPointer) + 2;

        // stack shrinks down, 2 bytes at a time
        self.set_register(Register::StackPointer, next_stack_pointer);
        self.stack_frame_size -= 2;

        self.memory.get_word(next_stack_pointer as usize)
    }

    fn push_state(&mut self) {
        // Push general purpose registers
        self.push(self.get_register(Register::Register1));
        self.push(self.get_register(Register::Register2));
        self.push(self.get_register(Register::Register3));
        self.push(self.get_register(Register::Register4));
        self.push(self.get_register(Register::Register5));
        self.push(self.get_register(Register::Register6));
        self.push(self.get_register(Register::Register7));
        self.push(self.get_register(Register::Register8));
        // Push instruciton pointer, which will be the return address
        self.push(self.get_register(Register::InstructionPointer));
        // Push stack size and +2 for this push
        let stack_size_to_save = self.stack_frame_size + 2;
        self.push(stack_size_to_save as u16);

        // Save the current stack pointer to frame pointer
        self.set_register(
            Register::FramePointer,
            self.get_register(Register::StackPointer),
        );

        // Reset stack size to 0
        self.stack_frame_size = 0;
    }

    fn pop_state(&mut self) {
        let stack_pointer_address = self.get_register(Register::FramePointer);

        // Rewind the stack pointer
        self.set_register(Register::StackPointer, stack_pointer_address);

        // Rewind stack size
        self.stack_frame_size = 2; // This is needed for the following pop, incase frame size is 0.
        let frame_size = self.pop();
        self.stack_frame_size = frame_size as usize;

        // Point the return address via instruction pointer
        let register_value = self.pop();
        self.set_register(Register::InstructionPointer, register_value);

        // Rewind the general purpose registers
        let register_value = self.pop();
        self.set_register(Register::Register8, register_value);
        let register_value = self.pop();
        self.set_register(Register::Register7, register_value);
        let register_value = self.pop();
        self.set_register(Register::Register6, register_value);
        let register_value = self.pop();
        self.set_register(Register::Register5, register_value);
        let register_value = self.pop();
        self.set_register(Register::Register4, register_value);
        let register_value = self.pop();
        self.set_register(Register::Register3, register_value);
        let register_value = self.pop();
        self.set_register(Register::Register2, register_value);
        let register_value = self.pop();
        self.set_register(Register::Register1, register_value);

        // Pop out argument list
        let n_args = self.pop();
        for _ in 0..n_args {
            self.pop();
        }

        // Rewind frame pointer
        let frame_pointer_address = stack_pointer_address + frame_size;
        self.set_register(Register::FramePointer, frame_pointer_address);
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
            Instruction::PushLit => {
                let value = self.fetch16();
                self.push(value);
            }
            Instruction::PushReg => {
                let index = self.fetch_register_index();
                let value = self.get_register_at(index);
                self.push(value);
            }
            Instruction::Pop => {
                let index = self.fetch_register_index();
                let value = self.pop();
                self.set_register_at(index, value);
            }
            Instruction::CalLit => {
                let address = self.fetch16();
                self.push_state();
                self.set_register(Register::InstructionPointer, address);
            }
            Instruction::CalReg => {
                let register_index = self.fetch_register_index();
                let address = self.get_register_at(register_index);
                self.push_state();
                self.set_register(Register::InstructionPointer, address);
            }
            Instruction::Ret => {
                self.pop_state();
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

#[derive(Debug, Clone, Copy)]
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
    StackPointer,
    FramePointer,
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
            10 => Register::StackPointer,
            11 => Register::FramePointer,
            _ => Register::None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Instruction {
    /// No operation for matching rest
    Noop = 0x00,
    /// Move a literal to a register
    MovLitReg = 0x10,
    /// Move the value in a register to another register
    MovRegReg = 0x11,
    /// Move the value in a register to a memory location
    MovRegMem = 0x12,
    /// Move the value in a memory location to a register
    MovMemReg = 0x13,
    /// Add the values in two registers and save it to the accumulator
    AddRegReg = 0x14,
    /// Jump to a memory location if the value is not equal to accumulator
    JmpNotEq = 0x15,
    /// Push a value to the stack
    PushLit = 0x17,
    /// Push the value in a register to the stack
    PushReg = 0x18,
    /// Pop the stack to the given register
    Pop = 0x1a,
    /// Call the subroutine at the literal
    CalLit = 0x5e,
    /// Call the subroutine at the register
    CalReg = 0x5f,
    /// Return from the subroutine
    Ret = 0x60,
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
            0x17 => Instruction::PushLit,
            0x18 => Instruction::PushReg,
            0x1a => Instruction::Pop,
            0x5e => Instruction::CalLit,
            0x5f => Instruction::CalReg,
            0x60 => Instruction::Ret,
            _ => Instruction::Noop,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Cpu, Instruction, Register};
    use crate::memory::Memory;

    fn assert_register_eq(cpu: &Cpu, register: &Register, value: u16, message: Option<&str>) {
        match message {
            Some(msg) => {
                assert_eq!(cpu.get_register(*register), value, "{}", msg);
            }
            None => {
                assert_eq!(
                    cpu.get_register(*register),
                    value,
                    "Register {:?} has the value {:#x}",
                    register,
                    value
                );
            }
        }
    }

    const TWO_BYTES: usize = 2;

    #[test]
    fn test_push_state_and_pop_state() {
        let memory = Memory::new(256);
        let mut cpu = Cpu::new(memory);
        let last_byte_pointer = cpu.memory.byte_length();

        cpu.set_register(Register::Register1, 0x1111);
        cpu.set_register(Register::Register2, 0x2222);
        cpu.set_register(Register::Register3, 0x3333);
        cpu.set_register(Register::Register4, 0x4444);
        cpu.set_register(Register::Register5, 0x5555);
        cpu.set_register(Register::Register6, 0x6666);
        cpu.set_register(Register::Register7, 0x7777);
        cpu.set_register(Register::Register8, 0x8888);

        cpu.push(0x4242); // Push argument 1 for the subroutine
        cpu.push(0x5252); // Push argument 2 for the subroutine
        cpu.push(0x0002); // Push number of arguments we sent to subroutine

        let stack_pointer_offset =
            1 * TWO_BYTES // Offsetted 2 bytes by default to start the stack
          + 3 * TWO_BYTES // Values pushed to the stack
        ;

        assert_register_eq(
            &cpu,
            &Register::StackPointer,
            (last_byte_pointer - stack_pointer_offset) as u16,
            Some("Stack pointer is pointing to the beginning"),
        );

        cpu.push_state();

        let stack_pointer_offset =
            1 * TWO_BYTES // Offsetted 2 bytes by default to start the stack
          + 3 * TWO_BYTES // Values pushed to the stack
          + 8 * TWO_BYTES // General purpose registers
          + 1 * TWO_BYTES // Instruction pointer
          + 1 * TWO_BYTES // Stack size
        ;

        assert_register_eq(
            &cpu,
            &Register::StackPointer,
            (last_byte_pointer - stack_pointer_offset) as u16,
            Some("Stack pointer is moved by the saved frame"),
        );

        assert_eq!(
            cpu.get_register(Register::FramePointer),
            cpu.get_register(Register::StackPointer)
        );

        assert_eq!(
            cpu.memory
                .get_word(cpu.get_register(Register::StackPointer) as usize + TWO_BYTES * 3),
            0x8888,
            "Pushed Register 8 to stack"
        );
        assert_eq!(
            cpu.memory
                .get_word(cpu.get_register(Register::StackPointer) as usize + TWO_BYTES * 7),
            0x4444,
            "Pushed Register 4 to stack"
        );

        cpu.pop_state();

        let stack_pointer_offset =
            1 * TWO_BYTES // Offsetted 2 bytes by default to start the stack
        ;

        assert_register_eq(
            &cpu,
            &Register::StackPointer,
            (last_byte_pointer - stack_pointer_offset) as u16,
            Some("Stack pointer is moved back to it's place"),
        );

        assert_register_eq(&cpu, &Register::Register3, 0x3333, None);
        assert_register_eq(&cpu, &Register::Register5, 0x5555, None);
    }

    #[test]
    fn test_push_and_pop() {
        let memory = Memory::new(256);
        let mut cpu = Cpu::new(memory);
        let last_byte_pointer = cpu.memory.byte_length();

        assert_register_eq(
            &cpu,
            &Register::StackPointer,
            (last_byte_pointer - 2) as u16,
            Some("Offset for the stack is 2 bytes before the last index"),
        );
        cpu.push(0x4243);
        assert_eq!(cpu.memory.get_byte(last_byte_pointer - 1), 0x43);
        assert_eq!(cpu.memory.get_byte(last_byte_pointer - 2), 0x42);
        assert_eq!(cpu.stack_frame_size, 2, "Stack grew two bytes");
        assert_register_eq(
            &cpu,
            &Register::StackPointer,
            (last_byte_pointer - 4) as u16,
            Some("Pointer points at the new empty address"),
        );

        let value = cpu.pop();
        assert_eq!(value, 0x4243);
        assert_eq!(cpu.stack_frame_size, 0, "Stack shrank two bytes");
        assert_eq!(
            cpu.memory.get_word(last_byte_pointer - 2),
            0x4243,
            "Shrinking stack doesn't need to remove the values."
        );
        assert_register_eq(
            &cpu,
            &Register::StackPointer,
            (last_byte_pointer - 2) as u16,
            Some("Pointer points at the new empty address"),
        );
    }

    #[test]
    fn bumps_instruction_pointer_at_every_step() {
        let memory = Memory::new(256);
        let mut cpu = Cpu::new(memory);

        assert_register_eq(&cpu, &Register::InstructionPointer, 0, None);
        cpu.step();
        assert_register_eq(&cpu, &Register::InstructionPointer, 1, None);
        cpu.step();
        assert_register_eq(&cpu, &Register::InstructionPointer, 2, None);
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

        assert_register_eq(&cpu, &Register::Register1, 0x1234, None);
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

        assert_register_eq(&cpu, &Register::Register2, 0x4243, None);
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

        assert_register_eq(&cpu, &Register::Register1, 0x1234, None);
        assert_register_eq(&cpu, &Register::Register2, 0xabcd, None);
        assert_register_eq(&cpu, &Register::Accumulator, 0xbe01, None);
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

        assert_register_eq(&cpu, &Register::Accumulator, 0x0003, None);
        assert_eq!(cpu.peek(0x0100), 0x0003);
    }

    #[test]
    fn pops_back_cpu_state_after_return() {
        let mut memory = Memory::new(256 * 256);

        // psh 0x1111
        // psh 0x2222
        // psh 0x3333
        //
        // mov 0x1234, r1
        // mov 0x5678, r4
        //
        // psh 0x0000   ;; number of arguments for this subroutine
        // cal my_subroutine:
        // psh 0x4444
        //
        // ;; at address 0x3000
        // my_subroutine:
        //  psh 0x0102
        //  psh 0x0304
        //  psh 0x0506
        //
        //  mov 0x0708, r1
        //  mov 0x0809, r8
        //  ret

        let subroutine_address: u16 = 0x0300;

        let mut i = 0;
        memory.set_byte(i, Instruction::PushLit as u8);
        i += 1;
        memory.set_byte(i, 0x11);
        i += 1;
        memory.set_byte(i, 0x11);
        i += 1;

        memory.set_byte(i, Instruction::PushLit as u8);
        i += 1;
        memory.set_byte(i, 0x22);
        i += 1;
        memory.set_byte(i, 0x22);
        i += 1;

        memory.set_byte(i, Instruction::PushLit as u8);
        i += 1;
        memory.set_byte(i, 0x33);
        i += 1;
        memory.set_byte(i, 0x33);
        i += 1;

        memory.set_byte(i, Instruction::MovLitReg as u8);
        i += 1;
        memory.set_byte(i, 0x12);
        i += 1;
        memory.set_byte(i, 0x34);
        i += 1;
        memory.set_byte(i, Register::Register1 as u8);
        i += 1;

        memory.set_byte(i, Instruction::MovLitReg as u8);
        i += 1;
        memory.set_byte(i, 0x56);
        i += 1;
        memory.set_byte(i, 0x78);
        i += 1;
        memory.set_byte(i, Register::Register4 as u8);
        i += 1;

        memory.set_byte(i, Instruction::PushLit as u8);
        i += 1;
        memory.set_byte(i, 0x00);
        i += 1;
        memory.set_byte(i, 0x00);
        i += 1;

        memory.set_byte(i, Instruction::CalLit as u8);
        i += 1;
        memory.set_byte(i, ((subroutine_address & 0xff00) >> 8) as u8);
        i += 1;
        memory.set_byte(i, (subroutine_address & 0x00ff) as u8);
        i += 1;

        memory.set_byte(i, Instruction::PushLit as u8);
        i += 1;
        memory.set_byte(i, 0x44);
        i += 1;
        memory.set_byte(i, 0x44);

        i = subroutine_address as usize;
        memory.set_byte(i, Instruction::PushLit as u8);
        i += 1;
        memory.set_byte(i, 0x01);
        i += 1;
        memory.set_byte(i, 0x02);
        i += 1;

        memory.set_byte(i, Instruction::PushLit as u8);
        i += 1;
        memory.set_byte(i, 0x03);
        i += 1;
        memory.set_byte(i, 0x04);
        i += 1;

        memory.set_byte(i, Instruction::PushLit as u8);
        i += 1;
        memory.set_byte(i, 0x05);
        i += 1;
        memory.set_byte(i, 0x06);
        i += 1;

        memory.set_byte(i, Instruction::MovLitReg as u8);
        i += 1;
        memory.set_byte(i, 0x07);
        i += 1;
        memory.set_byte(i, 0x08);
        i += 1;
        memory.set_byte(i, Register::Register1 as u8);
        i += 1;

        memory.set_byte(i, Instruction::MovLitReg as u8);
        i += 1;
        memory.set_byte(i, 0x09);
        i += 1;
        memory.set_byte(i, 0x0a);
        i += 1;
        memory.set_byte(i, Register::Register8 as u8);
        i += 1;

        memory.set_byte(i, Instruction::Ret as u8);

        let mut cpu = Cpu::new(memory);
        cpu.step_n(12);

        assert_register_eq(&cpu, &Register::Register1, 0x0708, None);
        assert_register_eq(&cpu, &Register::Register8, 0x090a, None);

        cpu.step_n(5);

        assert_register_eq(&cpu, &Register::Register1, 0x1234, None);
        assert_register_eq(&cpu, &Register::Register4, 0x5678, None);
        assert_register_eq(&cpu, &Register::Register8, 0x0000, None);
        assert_eq!(
            cpu.peek_stack().as_slice(),
            [0x12, 0x34, 0x44, 0x44, 0x33, 0x33, 0x22, 0x22, 0x11, 0x11]
        );
    }
}
