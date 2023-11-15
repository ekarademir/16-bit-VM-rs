mod cpu;
mod memory;

use crate::cpu::{Cpu, Instruction, Register};
use crate::memory::Memory;
use std::io::stdin;

fn main() {
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

    print_cpu(&cpu);

    loop {
        stdin().read_line(&mut (String::new())).unwrap();
        cpu.step();
        print_cpu(&cpu);
    }
}

fn print_cpu(cpu: &Cpu) {
    print_register(cpu, Register::InstructionPointer);
    print_register(cpu, Register::Accumulator);
    print_register(cpu, Register::Register1);
    print_register(cpu, Register::Register2);
    print_register(cpu, Register::Register3);
    print_register(cpu, Register::Register4);
    print_register(cpu, Register::Register5);
    print_register(cpu, Register::Register6);
    print_register(cpu, Register::Register7);
    print_register(cpu, Register::Register8);

    print_tape(cpu);
    print_stack(cpu);
}

fn print_register(cpu: &Cpu, register: Register) {
    println!("0x{:04x}  :: {:?}", cpu.peek_register(register), register);
}

fn print_stack(cpu: &Cpu) {
    let tape: Vec<u8> = cpu.peek_stack();
    let mut formatted: Vec<String> = Vec::new();
    for x in tape {
        formatted.push(format!("0x{:02x?}", x));
    }
    let joined = formatted.join(" ");
    println!(
        "Stack 0x{:04x} :: {}",
        cpu.peek_register(Register::StackPointer),
        joined
    );
}

fn print_tape(cpu: &Cpu) {
    let instruction_pointer = cpu.peek_register(Register::InstructionPointer);
    let tape: Vec<u8> = cpu.peek_tape(instruction_pointer as usize);
    let instruction: Instruction = if let Some(x) = tape.get(0) {
        (*x).into()
    } else {
        Instruction::Noop
    };
    let mut formatted: Vec<String> = Vec::new();
    for x in tape {
        formatted.push(format!("0x{:02x?}", x));
    }
    let joined = formatted.join(" ");
    println!(
        "Tape 0x{:04x} :: {} ::: {:?}",
        instruction_pointer, joined, instruction
    );
}
