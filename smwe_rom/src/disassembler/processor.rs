use crate::disassembler::{instruction::Instruction, opcodes::Mnemonic, registers::PRegister};

#[derive(Clone)]
pub struct Processor {
    pub stack: Vec<u8>,
    pub p_reg: PRegister,
}

impl Processor {
    pub fn new() -> Self {
        Self { stack: Vec::with_capacity(256), p_reg: PRegister(0b00110000) }
    }

    pub fn execute(&mut self, instr: Instruction) {
        use Mnemonic::*;
        match instr.opcode.mnemonic {
            SEP => self.p_reg.0 |= instr.operands()[0],
            REP => self.p_reg.0 &= !instr.operands()[0],
            PLP => {
                if let Some(&top) = self.stack.last() {
                    self.p_reg.0 = top;
                }
            }
            _ => {}
        }
    }
}
