use crate::disassembler::{instruction::Instruction, opcodes::Mnemonic::*, registers::*};

#[derive(Clone)]
pub struct Processor {
    pub p_reg: PRegister,
}

impl Processor {
    pub fn new() -> Self {
        Self { p_reg: PRegister(0b00110000) }
    }

    pub fn execute(&mut self, instr: Instruction) {
        match instr.opcode.mnemonic {
            SEP => {
                self.p_reg.0 |= instr.operands()[0];
            }
            REP => {
                self.p_reg.0 &= !instr.operands()[0];
            }
            _ => {}
        }
    }
}

impl Default for Processor {
    fn default() -> Self {
        Self::new()
    }
}
