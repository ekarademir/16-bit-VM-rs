mod cpu;
mod memory;

use crate::cpu::{Cpu, Instruction, Register};
use crate::memory::Memory;

fn main() {
    let mut memory = Memory::new(256 * 256);

    // start:
    //   mov #0x0100, r1
    //   mov 0x0001, r2
    //   add r1, r2
    //   mov acc, #0100
    //   jne 0x0003, start:

    let mut i = 0;
    memory.set_byte(i, Instruction::MovMemReg as u8);
    i += 1;
    memory.set_byte(i, 0x01);
    i += 1;
    memory.set_byte(i, 0x00);
    i += 1;
    memory.set_byte(i, Register::Register1 as u8);

    i += 1;
    memory.set_byte(i, Instruction::MovLitReg as u8);
    i += 1;
    memory.set_byte(i, 0x00);
    i += 1;
    memory.set_byte(i, 0x01);
    i += 1;
    memory.set_byte(i, Register::Register2 as u8);

    i += 1;
    memory.set_byte(i, Instruction::AddRegReg as u8);
    i += 1;
    memory.set_byte(i, Register::Register1 as u8);
    i += 1;
    memory.set_byte(i, Register::Register2 as u8);

    i += 1;
    memory.set_byte(i, Instruction::MovRegMem as u8);
    i += 1;
    memory.set_byte(i, Register::Accumulator as u8);
    i += 1;
    memory.set_byte(i, 0x01);
    i += 1;
    memory.set_byte(i, 0x00);

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

    println!("0x0100: {:?}", cpu.peek_tape(0x0100));
}
