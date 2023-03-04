use std::fmt::{Display, Formatter};

use smallvec::{smallvec, SmallVec};
use thiserror::Error;

use crate::{
    disassembler::{
        jump_tables::{EXECUTE_PTR_LONG_TRAMPOLINE_ADDR, EXECUTE_PTR_TRAMPOLINE_ADDR},
        opcodes::{AddressingMode::*, Mnemonic, Opcode, SNES_OPCODES},
        registers::PRegister,
    },
    snes_utils::addr::*,
};

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum InstructionParseError {
    #[error("No bytes provided to parse instruction from")]
    InputEmpty,
    #[error("Not enough bytes to read operands for instruction {0:08x}")]
    InputTooShort(u8),
}

// -------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Instruction {
    pub offset: AddrPc,
    pub opcode: Opcode,
    pub m_flag: bool,
    pub x_flag: bool,

    // Length might be shorter than 4, needs to be looked up by opcode
    operands: [u8; 4],
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct DisplayInstruction(Instruction);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct DisplayInstructionWithFlags(Instruction);

// -------------------------------------------------------------------------------------------------

impl Instruction {
    pub fn parse(bytes: &[u8], offset: AddrPc, p_reg: PRegister) -> Result<(Self, &[u8]), InstructionParseError> {
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

        Ok((Self { offset, opcode, operands, m_flag: p_reg.m_flag(), x_flag: p_reg.x_flag() }, rest))
    }

    pub fn display(self) -> DisplayInstruction {
        DisplayInstruction(self)
    }

    pub fn display_with_flags(self) -> DisplayInstructionWithFlags {
        DisplayInstructionWithFlags(self)
    }

    pub fn operands(&self) -> &[u8] {
        &self.operands[0..self.opcode.mode.operands_size()]
    }

    pub fn next_instructions(self) -> SmallVec<[AddrSnes; 2]> {
        use Mnemonic::*;

        let offset_snes = AddrSnes::try_from(self.offset).expect("Invalid instruction address");
        let is_jump_address_immediate = matches!(self.opcode.mode, Address | Long | Relative8 | Relative16);
        let next_instruction = offset_snes + self.opcode.instruction_size() as AddrInner;
        let maybe_jump_target = self.get_intermediate_address();

        match self.opcode.mnemonic {
            // Unconditional jumps and branches
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
            // Returns and interrupts
            RTS | RTL | RTI | BRK | COP => {
                // Interrupt handler destinations are read from internal header and enqueued at the start of disassembly.
                smallvec![]
            }
            _ => {
                if self.can_change_program_counter() {
                    log::error!("Unhandled branching instruction {self:?} at ${offset_snes:06X}");
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
        if matches!(self.opcode.mnemonic, JSR | JSL | BRK | COP) {
            let next_instruction = offset + self.opcode.instruction_size() as AddrInner;
            Some(next_instruction)
        } else {
            None
        }
    }

    fn get_intermediate_address(self) -> AddrSnes {
        let offset_snes = AddrSnes::try_from(self.offset).expect("Invalid instruction address");
        let op_bytes = self.operands();
        AddrSnes(match self.opcode.mode {
            m if (DirectPage..=DirectPageYIndex).contains(&m) => op_bytes[0] as AddrInner,
            DirectPageSIndex | DirectPageSIndexIndirectYIndex => op_bytes[0] as AddrInner,
            Address | AddressXIndex | AddressYIndex | AddressXIndexIndirect => {
                let bank = offset_snes.0 >> 16;
                let operand = u16::from_le_bytes([op_bytes[0], op_bytes[1]]);
                (bank << 16) | (operand as u32)
            }
            AddressIndirect | AddressLongIndirect => u16::from_le_bytes([op_bytes[0], op_bytes[1]]) as AddrInner,
            Long | LongXIndex => u32::from_le_bytes([op_bytes[0], op_bytes[1], op_bytes[2], 0]),
            Relative8 | Relative16 => {
                let operand_size = self.opcode.instruction_size() - 1;
                let program_counter = (offset_snes + 1usize + operand_size).0 as i32;
                let bank = program_counter >> 16;
                let jump_amount = match operand_size {
                    1 => op_bytes[0] as i8 as i32, // u8->i8 for the sign, i8->i32 for the size.
                    2 => i16::from_le_bytes([op_bytes[0], op_bytes[1]]) as i32,
                    _ => unreachable!(),
                };
                ((bank << 16) | (program_counter.wrapping_add(jump_amount) & 0xFFFF)) as AddrInner
            }
            _ => 0,
        })
    }

    pub fn can_change_program_counter(self) -> bool {
        self.opcode.mnemonic.can_change_program_counter()
    }

    pub fn is_single_path_leap(self) -> bool {
        self.opcode.mnemonic.is_single_path_leap()
    }

    pub fn is_double_path(self) -> bool {
        self.opcode.mnemonic.is_double_path()
    }

    pub fn is_branch_or_jump(self) -> bool {
        self.opcode.mnemonic.is_branch_or_jump()
    }

    pub fn is_subroutine_call(self) -> bool {
        self.opcode.mnemonic.is_subroutine_call()
    }

    pub fn is_subroutine_return(self) -> bool {
        self.opcode.mnemonic.is_subroutine_return()
    }

    pub fn uses_jump_table(self) -> bool {
        self.next_instructions()
            .iter()
            .any(|&t| t == EXECUTE_PTR_TRAMPOLINE_ADDR || t == EXECUTE_PTR_LONG_TRAMPOLINE_ADDR)
    }
}

impl Display for DisplayInstructionWithFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}{}] ", ['m', 'M'][self.0.m_flag as usize], ['x', 'X'][self.0.x_flag as usize])?;
        self.0.display().fmt(f)
    }
}

impl Display for DisplayInstruction {
    fn fmt(&self, outer_fmt: &mut Formatter) -> std::fmt::Result {
        use std::io::Write;
        let mut fmt: SmallVec<[u8; 64]> = Default::default();
        let Instruction { x_flag, m_flag, .. } = self.0;

        let (address_long, address_short, address_dp) = {
            let a = self.0.get_intermediate_address();
            (a.0, a.absolute(), a.low())
        };

        write!(fmt, "{}", self.0.opcode.mnemonic).unwrap();
        match self.0.opcode.mode {
            Implied => {
                // no-op
            }
            Accumulator => {
                write!(fmt, " A").unwrap();
            }
            Constant8 | Immediate8 => {
                write!(fmt, " #${:02X}", self.0.operands[0]).unwrap();
            }
            Immediate16 => {
                write!(fmt, " #${:04X}", u16::from_le_bytes([self.0.operands[0], self.0.operands[1]])).unwrap();
            }
            ImmediateXFlagDependent | ImmediateMFlagDependent => {
                let x = self.0.opcode.mode == ImmediateXFlagDependent && x_flag;
                let m = self.0.opcode.mode == ImmediateMFlagDependent && m_flag;
                if x || m {
                    write!(fmt, " #${:02X}", self.0.operands[0]).unwrap();
                } else {
                    write!(fmt, " #${:04X}", u16::from_le_bytes([self.0.operands[0], self.0.operands[1]])).unwrap();
                }
            }
            DirectPage => {
                write!(fmt, " ${address_dp:02X}").unwrap();
            }
            Relative8 => {
                let address = self.0.operands[0] as u32;
                let address = address & !(-1i32 << 8) as u32;
                write!(fmt, " ${address:02X}").unwrap();
            }
            Relative16 => {
                let address = u16::from_le_bytes([self.0.operands[0], self.0.operands[1]]) as u32;
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
                write!(fmt, " ${:02X}, ${:02X}", self.0.operands[0], self.0.operands[1]).unwrap();
            }
        };
        outer_fmt.pad(std::str::from_utf8(&fmt).unwrap())
    }
}
