mod cpu;
mod memory;

use crate::cpu::{Cpu, Instruction, Register};
use crate::memory::Memory;

fn main() {
    let mut memory = Memory::new(32);

    // mov 0x1234, r1
    // mov 0xabcd, r2
    // add r1, r2

    let mut i = 0;
    memory.set_byte(i, Instruction::MovLitR1 as u8);
    i += 1;
    memory.set_byte(i, 0x12);
    i += 1;
    memory.set_byte(i, 0x34);
    i += 1;

    memory.set_byte(i, Instruction::MovLitR2 as u8);
    i += 1;
    memory.set_byte(i, 0xab);
    i += 1;
    memory.set_byte(i, 0xcd);
    i += 1;

    memory.set_byte(i, Instruction::AddRegReg as u8);
    i += 1;
    memory.set_byte(i, Register::Register1 as u8);
    i += 1;
    memory.set_byte(i, Register::Register2 as u8);

    let mut cpu = Cpu::new(memory);
    cpu.step_n(3);

    println!("{:?}", cpu);
}
