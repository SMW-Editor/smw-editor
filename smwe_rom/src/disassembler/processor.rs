use crate::disassembler::{instruction::Instruction, opcodes::Mnemonic::*, registers::*};

#[derive(Clone)]
pub struct Processor {
    pub p_reg: PRegister,
    pub stack: Vec<u8>,
}

impl Processor {
    pub fn new() -> Self {
        Self { p_reg: PRegister(0b00110000), stack: Vec::with_capacity(256) }
    }

    pub fn execute(&mut self, instr: Instruction) {
        match instr.opcode.mnemonic {
            SEP => self.p_reg.0 |= instr.operands()[0],
            REP => self.p_reg.0 &= !instr.operands()[0],
            PHP => self.stack.push(self.p_reg.0),
            PLP => match self.stack.pop() {
                Some(p) => self.p_reg.0 = p,
                None => log::error!("Stack underflow at {} ({:?})", instr.offset, instr.display()),
            },
            _ => {}
        }
    }
}

impl Default for Processor {
    fn default() -> Self {
        Self::new()
    }
}
