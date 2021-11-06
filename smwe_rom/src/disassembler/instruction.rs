use std::fmt::{Display, Write};

use crate::{
    disassembler::opcodes::{AddressingMode::*, Mnemonic, Opcode, SNES_OPCODES},
    error::InstructionParseError,
    snes_utils::addr::*,
};

// -------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Instruction {
    pub opcode: Opcode,
    // Length might be shorter than 4, needs to be looked up by opcode
    operands:   [u8; 4],
}

#[derive(Copy, Clone)]
pub struct DisplayInstruction {
    i:      Instruction,
    offset: usize,
    x_flag: bool,
    m_flag: bool,
}

// -------------------------------------------------------------------------------------------------

impl Instruction {
    pub fn parse(bytes: &[u8]) -> Result<(Instruction, &[u8]), InstructionParseError> {
        let (&opcode_raw, rest) = bytes.split_first().ok_or(InstructionParseError::InputEmpty)?;
        let opcode = SNES_OPCODES[opcode_raw as usize];
        let operands_size = opcode.mode.operands_size();
        if rest.len() < operands_size {
            return Err(InstructionParseError::InputTooShort(opcode_raw));
        }
        let (operands_v, rest) = rest.split_at(operands_size);
        let mut operands = [0u8; 4];
        operands[..operands_v.len()].copy_from_slice(operands_v);
        Ok((Self { opcode, operands }, rest))
    }

    pub fn display(self, offset: usize, x_flag: bool, m_flag: bool) -> DisplayInstruction {
        DisplayInstruction { i: self, offset, x_flag, m_flag }
    }

    pub fn operands(&self) -> &[u8] {
        &self.operands[0..self.opcode.mode.operands_size()]
    }

    fn get_intermediate_address(self, offset: usize, resolve: bool) -> u32 {
        let op_bytes = self.operands();
        match self.opcode.mode {
            m if (DirectPage..=DirectPageYIndex).contains(&m) => {
                if resolve {
                    let operand = op_bytes[0] as u32;
                    let direct_page: u32 = todo!();
                    ((direct_page + operand) & 0xFFFF) as u32
                } else {
                    op_bytes[0] as u32
                }
            }
            DirectPageSIndex | DirectPageSIndexIndirectYIndex => op_bytes[0] as u32,
            Address | AddressXIndex | AddressYIndex | AddressXIndexIndirect => {
                let bank = if self.opcode.mnemonic == Mnemonic::JSR || self.opcode.mnemonic == Mnemonic::JMP {
                    let offset_pc = AddrPc(offset);
                    (AddrSnes::try_from_lorom(offset_pc).unwrap().0 >> 16) as u32
                } else {
                    //TODO
                    0xDEAD
                };
                let operand = u16::from_le_bytes([op_bytes[0], op_bytes[1]]);
                (bank << 16) | (operand as u32)
            }
            AddressIndirect | AddressLongIndirect => u16::from_le_bytes([op_bytes[0], op_bytes[1]]) as u32,
            Long | LongXIndex => u32::from_le_bytes([op_bytes[0], op_bytes[1], op_bytes[2], 0]),
            Relative8 | Relative16 => {
                let operand_size = self.opcode.instruction_size() - 1;
                let program_counter = {
                    let offset_pc = AddrPc(offset + 1 + operand_size);
                    AddrSnes::try_from_lorom(offset_pc).unwrap().0 as u32
                };
                let bank = program_counter >> 16;
                let address = if self.opcode.mode == Relative8 {
                    op_bytes[0] as u32
                } else {
                    u16::from_le_bytes([op_bytes[0], op_bytes[1]]) as u32
                };

                (bank << 16) | ((program_counter + address) & 0xFFFF)
            }
            _ => 0,
        }
    }
}

impl Display for DisplayInstruction {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let DisplayInstruction { i, x_flag, m_flag, offset } = *self;
        let address = i.get_intermediate_address(offset, false);
        write!(fmt, "{}", i.opcode.mnemonic)?;
        match i.opcode.mode {
            Implied => {
                // no-op
                Ok(())
            }
            Accumulator => fmt.write_char('A'),
            Constant8 | Immediate8 => {
                write!(fmt, " #${:02X}", i.operands[0])
            }
            Immediate16 => {
                write!(fmt, " #${:04X}", u16::from_le_bytes([i.operands[0], i.operands[1]]))
            }
            ImmediateXFlagDependent | ImmediateMFlagDependent => {
                let x = i.opcode.mode == ImmediateXFlagDependent && x_flag;
                let m = i.opcode.mode == ImmediateMFlagDependent && m_flag;
                if x || m {
                    write!(fmt, " #${:02X}", i.operands[0])
                } else {
                    write!(fmt, " #${:04X}", u16::from_le_bytes([i.operands[0], i.operands[1]]))
                }
            }
            DirectPage => {
                write!(fmt, " ${:02X}", address)
            }
            Relative8 => {
                let address = i.operands[0] as u32;
                let address = address & !(-1 << 8) as u32;
                write!(fmt, " ${:02X}", address)
            }
            Relative16 => {
                let address = u16::from_le_bytes([i.operands[0], i.operands[1]]) as u32;
                let address = address & !(-1 << 16) as u32;
                write!(fmt, " ${:04X}", address)
            }
            Address => {
                write!(fmt, " ${:04X}", address)
            }
            Long => {
                write!(fmt, " ${:06X}", address)
            }
            DirectPageXIndex | AddressXIndex | LongXIndex => {
                write!(fmt, " ${:02X}, X", address)
            }
            DirectPageYIndex | AddressYIndex => {
                write!(fmt, " ${:02X}, Y", address)
            }
            DirectPageSIndex => {
                write!(fmt, " ${:02X}, S", address)
            }
            DirectPageIndirect => {
                write!(fmt, " (${:02X})", address)
            }
            AddressIndirect => {
                write!(fmt, " (${:04X})", address)
            }
            DirectPageXIndexIndirect => {
                write!(fmt, " (${:02X}, X)", address)
            }
            AddressXIndexIndirect => {
                write!(fmt, " (${:04X}, X)", address)
            }
            DirectPageIndirectYIndex => {
                write!(fmt, " (${:02X}), Y", address)
            }
            DirectPageSIndexIndirectYIndex => {
                write!(fmt, " (${:02X}, S), Y", address)
            }
            DirectPageLongIndirect => {
                write!(fmt, " [${:02X}]", address)
            }
            AddressLongIndirect => {
                write!(fmt, " [${:04X}]", address)
            }
            DirectPageLongIndirectYIndex => {
                write!(fmt, " [${:02X}], Y", address)
            }
            BlockMove => {
                write!(fmt, " ${:02X}, ${:02X}", i.operands[0], i.operands[1])
            }
        }
    }
}
