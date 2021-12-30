use crate::disassembler::{instruction::Instruction, opcodes::Mnemonic::*, registers::*};

#[derive(Clone)]
pub struct Processor {
    pub stack:  Vec<u8>,
    pub dp_reg: DPRegister,
    pub p_reg:  PRegister,
}

impl Processor {
    pub fn new() -> Self {
        Self { stack: Vec::with_capacity(0x2000), dp_reg: DPRegister(0), p_reg: PRegister(0b00110000) }
    }

    pub fn execute(&mut self, instr: Instruction) {
        match instr.opcode.mnemonic {
            SEP => {
                self.p_reg.0 |= instr.operands()[0];
            }
            REP => {
                self.p_reg.0 &= !instr.operands()[0];
            }
            PHP => {
                self.stack.push(self.p_reg.0);
            }
            PLP => {
                if let Some(top) = self.stack.pop() {
                    self.p_reg.0 = top;
                }
            }
            RTI => {}
            XCE => {}
            _ => {}
        }
    }
}

impl Default for Processor {
    fn default() -> Self {
        Self::new()
    }
}
