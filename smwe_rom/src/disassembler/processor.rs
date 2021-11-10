use crate::disassembler::{instruction::Instruction, opcodes::Mnemonic::*, registers::*};

#[derive(Clone)]
pub struct Processor {
    pub stack:  Vec<u8>,
    pub a_reg:  ARegister,
    pub x_reg:  XRegister,
    pub y_reg:  YRegister,
    pub db_reg: DBRegister,
    pub dp_reg: DPRegister,
    pub pb_reg: PBRegister,
    pub pc_reg: PCRegister,
    pub p_reg:  PRegister,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            stack:  Vec::with_capacity(256),
            a_reg:  ARegister(0),
            x_reg:  XRegister(0),
            y_reg:  YRegister(0),
            db_reg: DBRegister(0),
            dp_reg: DPRegister(0),
            pb_reg: PBRegister(0),
            pc_reg: PCRegister(0),
            p_reg:  PRegister(0b00110000),
        }
    }

    pub fn execute(&mut self, instr: Instruction) {
        let ops = instr.operands();
        match instr.opcode.mnemonic {
            SEP => {
                self.p_reg.0 |= instr.operands()[0];
            }
            REP => {
                self.p_reg.0 &= !instr.operands()[0];
            }
            PEA => {
                self.stack.push(ops[1]);
                self.stack.push(ops[0]);
            }
            PEI => {
                // TODO - fetch u16 from memory: ops contains an indirect address
                self.push_word(0);
            }
            PER => {
                // TODO - fetch u16 from memory: ops contains an address relative to Program Counter
                self.push_word(0);
            }
            PHA => {
                if self.p_reg.m_flag() {
                    self.stack.push(self.a_reg.0 as u8)
                } else {
                    self.push_word(self.a_reg.0);
                }
            }
            PHX => {
                if self.p_reg.x_flag() {
                    self.stack.push(self.x_reg.0 as u8);
                } else {
                    self.push_word(self.x_reg.0);
                }
            }
            PHY => {
                if self.p_reg.x_flag() {
                    self.stack.push(self.y_reg.0 as u8);
                } else {
                    self.push_word(self.y_reg.0);
                }
            }
            PHD => {
                self.push_word(self.dp_reg.0);
            }
            PHK => {
                self.stack.push(self.pb_reg.0);
            }
            PLA => {
                if self.p_reg.m_flag() && self.stack.len() >= 1 {
                    self.a_reg.0 = self.stack.pop().unwrap() as u16;
                } else if self.stack.len() >= 2 {
                    self.a_reg.0 = self.pull_word();
                }
            }
            PLX => {
                if self.p_reg.x_flag() && self.stack.len() >= 1 {
                    self.x_reg.0 = self.stack.pop().unwrap() as u16;
                } else if self.stack.len() >= 2 {
                    self.x_reg.0 = self.pull_word();
                }
            }
            PLY => {
                if self.p_reg.x_flag() && self.stack.len() >= 1 {
                    self.y_reg.0 = self.stack.pop().unwrap() as u16;
                } else if self.stack.len() >= 2 {
                    self.y_reg.0 = self.pull_word();
                }
            }
            PLD => {
                if self.stack.len() >= 2 {
                    self.dp_reg.0 = self.pull_word();
                }
            }
            PLP => {
                if let Some(top) = self.stack.pop() {
                    self.p_reg.0 = top;
                }
            }
            _ => {}
        }
    }

    fn push_word(&mut self, word: u16) {
        let [byte_1, byte_2] = word.to_le_bytes();
        self.stack.push(byte_1);
        self.stack.push(byte_2);
    }

    fn pull_word(&mut self) -> u16 {
        let byte_2 = self.stack.pop().unwrap() as u16;
        let byte_1 = (self.stack.pop().unwrap() as u16) << 8;
        byte_1 | byte_2
    }
}
