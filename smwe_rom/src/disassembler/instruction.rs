use std::fmt::Display;

use smallvec::{smallvec, SmallVec};

use crate::{
    disassembler::{
        opcodes::{AddressingMode::*, Mnemonic, Opcode, SNES_OPCODES},
        registers::PRegister,
    },
    error::InstructionParseError,
    snes_utils::addr::*,
};

// -------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Instruction {
    pub opcode: Opcode,
    // Length might be shorter than 4, needs to be looked up by opcode
    operands:   [u8; 4],
}

#[derive(Copy, Clone)]
pub struct DisplayInstruction {
    i:      Instruction,
    offset: AddrPc,
    x_flag: bool,
    m_flag: bool,
}

// -------------------------------------------------------------------------------------------------

impl Instruction {
    pub fn parse(bytes: &[u8], p_reg: PRegister) -> Result<(Self, &[u8]), InstructionParseError> {
        let (&opcode_raw, rest) = bytes.split_first().ok_or(InstructionParseError::InputEmpty)?;
        let mut opcode = SNES_OPCODES[opcode_raw as usize];

        if opcode.mode == ImmediateMFlagDependent {
            opcode.mode = if p_reg.m_flag() { Immediate8 } else { Immediate16 };
        } else if opcode.mode == ImmediateXFlagDependent {
            opcode.mode = if p_reg.x_flag() { Immediate8 } else { Immediate16 };
        }

        let operands_size = opcode.mode.operands_size();
        if rest.len() < operands_size {
            return Err(InstructionParseError::InputTooShort(opcode_raw));
        }
        let (operands_v, rest) = rest.split_at(operands_size);
        let mut operands = [0u8; 4];
        operands[..operands_v.len()].copy_from_slice(operands_v);

        Ok((Self { opcode, operands }, rest))
    }

    pub fn display(self, offset: AddrPc, x_flag: bool, m_flag: bool) -> DisplayInstruction {
        DisplayInstruction { i: self, offset, x_flag, m_flag }
    }

    pub fn operands(&self) -> &[u8] {
        &self.operands[0..self.opcode.mode.operands_size()]
    }

    pub fn next_instructions(self, offset: AddrSnes) -> SmallVec<[AddrSnes; 2]> {
        use Mnemonic::*;

        let is_jump_address_immediate = [Address, Long, Relative8, Relative16].contains(&self.opcode.mode);
        let next_instruction = offset + self.opcode.instruction_size();
        let maybe_jump_target = self.get_intermediate_address(offset);

        match self.opcode.mnemonic {
            // Jumps & subroutines
            BRA | BRL | JMP | JML | JSR | JSL => {
                if is_jump_address_immediate {
                    smallvec![maybe_jump_target]
                } else {
                    smallvec![]
                }
            }
            // Conditional branches
            BCC | BCS | BEQ | BMI | BNE | BPL | BVC | BVS => {
                if is_jump_address_immediate {
                    smallvec![maybe_jump_target, next_instruction]
                } else {
                    smallvec![next_instruction]
                }
            }
            // Returns
            RTS | RTL | RTI => {
                smallvec![]
            }
            // Interrupts
            BRK | COP => {
                // todo: interrupt handler destination
                smallvec![]
            }
            _ => {
                if self.opcode.mnemonic.can_branch() {
                    log::error!("Unhandled branching instruction {self:?} at ${offset:06X}");
                    smallvec![]
                } else {
                    smallvec![next_instruction]
                }
            }
        }
    }

    /// If the instruction leads to a subroutine that would return flow control, gets the returned flow control address.
    pub fn return_instruction(self, offset: AddrSnes) -> Option<AddrSnes> {
        use Mnemonic::*;

        // Returning jumps and interrupts
        if [JSR, JSL, BRK, COP].contains(&self.opcode.mnemonic) {
            let next_instruction = offset + self.opcode.instruction_size();
            Some(next_instruction)
        } else {
            None
        }
    }

    fn get_intermediate_address(self, offset: AddrSnes) -> AddrSnes {
        let op_bytes = self.operands();
        AddrSnes(match self.opcode.mode {
            m if (DirectPage..=DirectPageYIndex).contains(&m) => op_bytes[0] as u32,
            DirectPageSIndex | DirectPageSIndexIndirectYIndex => op_bytes[0] as u32,
            Address | AddressXIndex | AddressYIndex | AddressXIndexIndirect => {
                let bank = (offset.0 >> 16) as u32;
                let operand = u16::from_le_bytes([op_bytes[0], op_bytes[1]]);
                (bank << 16) | (operand as u32)
            }
            AddressIndirect | AddressLongIndirect => u16::from_le_bytes([op_bytes[0], op_bytes[1]]) as u32,
            Long | LongXIndex => u32::from_le_bytes([op_bytes[0], op_bytes[1], op_bytes[2], 0]),
            Relative8 | Relative16 => {
                let operand_size = self.opcode.instruction_size() - 1;
                let program_counter = (offset + 1 + operand_size).0 as i32;
                let bank = program_counter >> 16;
                let jump_amount = match operand_size {
                    1 => op_bytes[0] as i8 as i32, // u8->i8 for the sign, i8->i32 for the size.
                    2 => i16::from_le_bytes([op_bytes[0], op_bytes[1]]) as i32,
                    _ => unreachable!(),
                };
                ((bank << 16) | (program_counter.wrapping_add(jump_amount) & 0xFFFF)) as u32
            }
            _ => 0,
        } as usize)
    }
}

impl Display for DisplayInstruction {
    fn fmt(&self, outer_fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::io::Write;
        let mut fmt: SmallVec<[u8; 64]> = Default::default();
        let DisplayInstruction { i, x_flag, m_flag, offset } = *self;

        let offset = AddrSnes::try_from_lorom(offset).unwrap_or_default();
        let address_long = i.get_intermediate_address(offset).0;
        let address_short = address_long & 0xFFFF;
        let address_dp = address_long & 0xFF;

        write!(fmt, "{}", i.opcode.mnemonic).unwrap();
        match i.opcode.mode {
            Implied => {
                // no-op
            }
            Accumulator => {
                write!(fmt, " A").unwrap();
            }
            Constant8 | Immediate8 => {
                write!(fmt, " #${:02X}", i.operands[0]).unwrap();
            }
            Immediate16 => {
                write!(fmt, " #${:04X}", u16::from_le_bytes([i.operands[0], i.operands[1]])).unwrap();
            }
            ImmediateXFlagDependent | ImmediateMFlagDependent => {
                let x = i.opcode.mode == ImmediateXFlagDependent && x_flag;
                let m = i.opcode.mode == ImmediateMFlagDependent && m_flag;
                if x || m {
                    write!(fmt, " #${:02X}", i.operands[0]).unwrap();
                } else {
                    write!(fmt, " #${:04X}", u16::from_le_bytes([i.operands[0], i.operands[1]])).unwrap();
                }
            }
            DirectPage => {
                write!(fmt, " ${address_dp:02X}").unwrap();
            }
            Relative8 => {
                let address = i.operands[0] as u32;
                let address = address & !(-1i32 << 8) as u32;
                write!(fmt, " ${address:02X}").unwrap();
            }
            Relative16 => {
                let address = u16::from_le_bytes([i.operands[0], i.operands[1]]) as u32;
                let address = address & !(-1i32 << 16) as u32;
                write!(fmt, " ${address:04X}").unwrap();
            }
            Address => {
                write!(fmt, " ${address_short:04X}").unwrap();
            }
            Long => {
                write!(fmt, " ${address_long:06X}").unwrap();
            }
            DirectPageXIndex | AddressXIndex | LongXIndex => {
                write!(fmt, " ${address_dp:02X}, X").unwrap();
            }
            DirectPageYIndex | AddressYIndex => {
                write!(fmt, " ${address_dp:02X}, Y").unwrap();
            }
            DirectPageSIndex => {
                write!(fmt, " ${address_dp:02X}, S").unwrap();
            }
            DirectPageIndirect => {
                write!(fmt, " (${address_dp:02X})").unwrap();
            }
            AddressIndirect => {
                write!(fmt, " (${address_short:04X})").unwrap();
            }
            DirectPageXIndexIndirect => {
                write!(fmt, " (${address_dp:02X}, X)").unwrap();
            }
            AddressXIndexIndirect => {
                write!(fmt, " (${address_short:04X}, X)").unwrap();
            }
            DirectPageIndirectYIndex => {
                write!(fmt, " (${address_dp:02X}), Y").unwrap();
            }
            DirectPageSIndexIndirectYIndex => {
                write!(fmt, " (${address_dp:02X}, S), Y").unwrap();
            }
            DirectPageLongIndirect => {
                write!(fmt, " [${address_dp:02X}]").unwrap();
            }
            AddressLongIndirect => {
                write!(fmt, " [${address_short:04X}]").unwrap();
            }
            DirectPageLongIndirectYIndex => {
                write!(fmt, " [${address_dp:02X}], Y").unwrap();
            }
            BlockMove => {
                write!(fmt, " ${:02X}, ${:02X}", i.operands[0], i.operands[1]).unwrap();
            }
        };
        outer_fmt.pad(std::str::from_utf8(&fmt).unwrap())
    }
}
