use std::fmt::Display;

use smallvec::{smallvec, SmallVec};

use crate::{
    disassembler::{
        opcodes::{AddressingMode, AddressingMode::*, Mnemonic, Opcode, SNES_OPCODES},
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
    i:           Instruction,
    offset:      AddrPc,
    x_flag:      bool,
    m_flag:      bool,
    direct_page: u16,
    data_bank:   u8,
}

// -------------------------------------------------------------------------------------------------

impl Instruction {
    pub fn parse(bytes: &[u8], p_reg: PRegister) -> Result<(Self, &[u8]), InstructionParseError> {
        let (&opcode_raw, rest) = bytes.split_first().ok_or(InstructionParseError::InputEmpty)?;
        let mut opcode = SNES_OPCODES[opcode_raw as usize];

        if opcode.mode == AddressingMode::ImmediateMFlagDependent {
            opcode.mode = if p_reg.m_flag() { AddressingMode::Immediate8 } else { AddressingMode::Immediate16 };
        } else if opcode.mode == AddressingMode::ImmediateXFlagDependent {
            opcode.mode = if p_reg.x_flag() { AddressingMode::Immediate8 } else { AddressingMode::Immediate16 };
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

    pub fn display(
        self, offset: AddrPc, x_flag: bool, m_flag: bool, direct_page: u16, data_bank: u8,
    ) -> DisplayInstruction {
        DisplayInstruction { i: self, offset, x_flag, m_flag, direct_page, data_bank }
    }

    pub fn operands(&self) -> &[u8] {
        &self.operands[0..self.opcode.mode.operands_size()]
    }

    pub fn next_instructions(self, offset: AddrSnes, direct_page: u16, data_bank: u8) -> SmallVec<[AddrSnes; 2]> {
        use AddressingMode::*;
        use Mnemonic::*;

        let is_jump_address_immediate = [Address, Long, Relative8, Relative16].contains(&self.opcode.mode);
        let next_instruction = offset + self.opcode.instruction_size();
        let maybe_jump_target = self.get_intermediate_address(offset, direct_page, data_bank, true);

        match self.opcode.mnemonic {
            // Unconditional jumps (single path)
            BRA | BRL | JMP | JML | JSR | JSL => {
                if is_jump_address_immediate {
                    smallvec![maybe_jump_target]
                } else {
                    smallvec![]
                }
            }
            // Conditional and returning jumps (2 paths)
            BCC | BCS | BEQ | BMI | BNE | BPL | BVC | BVS => {
                if is_jump_address_immediate {
                    smallvec![next_instruction, maybe_jump_target]
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
                smallvec![next_instruction]
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

    fn get_intermediate_address(self, offset: AddrSnes, direct_page: u16, data_bank: u8, resolve: bool) -> AddrSnes {
        let op_bytes = self.operands();
        AddrSnes(match self.opcode.mode {
            m if (DirectPage..=DirectPageYIndex).contains(&m) => {
                if resolve {
                    let operand = op_bytes[0] as u32;
                    (direct_page as u32 + operand) & 0xFFFF
                } else {
                    op_bytes[0] as u32
                }
            }
            DirectPageSIndex | DirectPageSIndexIndirectYIndex => op_bytes[0] as u32,
            Address | AddressXIndex | AddressYIndex | AddressXIndexIndirect => {
                use Mnemonic::*;
                let bank =
                    if [JSR, JMP].contains(&self.opcode.mnemonic) { (offset.0 >> 16) as u32 } else { data_bank as u32 };
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
        let DisplayInstruction { i, x_flag, m_flag, offset, direct_page, data_bank } = *self;

        let offset = AddrSnes::try_from_lorom(offset).unwrap_or_default();
        let address = i.get_intermediate_address(offset, direct_page, data_bank, false).0;

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
                write!(fmt, " ${address:02X}").unwrap();
            }
            Relative8 => {
                let address = i.operands[0] as u32;
                let address = address & !(-1 << 8) as u32;
                write!(fmt, " ${address:02X}").unwrap();
            }
            Relative16 => {
                let address = u16::from_le_bytes([i.operands[0], i.operands[1]]) as u32;
                let address = address & !(-1 << 16) as u32;
                write!(fmt, " ${address:04X}").unwrap();
            }
            Address => {
                write!(fmt, " ${address:04X}").unwrap();
            }
            Long => {
                write!(fmt, " ${address:06X}").unwrap();
            }
            DirectPageXIndex | AddressXIndex | LongXIndex => {
                write!(fmt, " ${address:02X}, X").unwrap();
            }
            DirectPageYIndex | AddressYIndex => {
                write!(fmt, " ${address:02X}, Y").unwrap();
            }
            DirectPageSIndex => {
                write!(fmt, " ${address:02X}, S").unwrap();
            }
            DirectPageIndirect => {
                write!(fmt, " (${address:02X})").unwrap();
            }
            AddressIndirect => {
                write!(fmt, " (${address:04X})").unwrap();
            }
            DirectPageXIndexIndirect => {
                write!(fmt, " (${address:02X}, X)").unwrap();
            }
            AddressXIndexIndirect => {
                write!(fmt, " (${address:04X}, X)").unwrap();
            }
            DirectPageIndirectYIndex => {
                write!(fmt, " (${address:02X}), Y").unwrap();
            }
            DirectPageSIndexIndirectYIndex => {
                write!(fmt, " (${address:02X}, S), Y").unwrap();
            }
            DirectPageLongIndirect => {
                write!(fmt, " [${address:02X}]").unwrap();
            }
            AddressLongIndirect => {
                write!(fmt, " [${address:04X}]").unwrap();
            }
            DirectPageLongIndirectYIndex => {
                write!(fmt, " [${address:02X}], Y").unwrap();
            }
            BlockMove => {
                write!(fmt, " ${:02X}, ${:02X}", i.operands[0], i.operands[1]).unwrap();
            }
        };
        outer_fmt.pad(std::str::from_utf8(&fmt).unwrap())
    }
}
