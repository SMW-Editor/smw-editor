use crate::{
    disassembler::opcodes::{
        AddressingMode::*,
        Mnemonic,
        Opcode,
    },
    snes_utils::addr::*,
};

// -------------------------------------------------------------------------------------------------

pub struct Instruction {
    pub opcode: Opcode,
    pub operands: Vec<u8>,
}

// -------------------------------------------------------------------------------------------------

impl Instruction {
    pub fn format_to_string(&self, offset: usize, x_flag: bool, m_flag: bool) -> String {
        let address = self.get_intermediate_address(offset, &self.operands, false);
        format!("{}{}", self.opcode.mnemonic, match self.opcode.mode {
            Implied => {
                String::new()
            }
            Accumulator => {
                String::from(" A")
            }
            Constant8 | Immediate8 => {
                format!(" #${:02X}", self.operands[0])
            }
            Immediate16 => {
                format!(" #${:04X}", u16::from_le_bytes([self.operands[0], self.operands[1]]))
            }
            ImmediateXFlagDependent | ImmediateMFlagDependent => {
                let x = self.opcode.mode == ImmediateXFlagDependent && x_flag;
                let m = self.opcode.mode == ImmediateMFlagDependent && m_flag;
                if x || m {
                    format!(" #${:02X}", self.operands[0])
                } else {
                    format!(" #${:04X}", u16::from_le_bytes([self.operands[0], self.operands[1]]))
                }
            }
            DirectPage => {
                format!(" ${:02X}", address)
            }
            Relative8 => {
                let address = self.operands[0] as u32;
                let address = address & !(-1 << 8) as u32;
                format!(" ${:02X}", address)
            }
            Relative16 => {
                let address = u16::from_le_bytes([self.operands[0], self.operands[1]]) as u32;
                let address = address & !(-1 << 16) as u32;
                format!(" ${:04X}", address)
            }
            Address => {
                format!(" ${:04X}", address)
            }
            Long => {
                format!(" ${:06X}", address)
            }
            DirectPageXIndex | AddressXIndex | LongXIndex => {
                format!(" ${:02X}, X", address)
            }
            DirectPageYIndex | AddressYIndex => {
                format!(" ${:02X}, Y", address)
            }
            DirectPageSIndex => {
                format!(" ${:02X}, S", address)
            }
            DirectPageIndirect => {
                format!(" (${:02X})", address)
            }
            AddressIndirect => {
                format!(" (${:04X})", address)
            }
            DirectPageXIndexIndirect => {
                format!(" (${:02X}, X)", address)
            }
            AddressXIndexIndirect => {
                format!(" (${:04X}, X)", address)
            }
            DirectPageIndirectYIndex => {
                format!(" (${:02X}), Y", address)
            }
            DirectPageSIndexIndirectYIndex => {
                format!(" (${:02X}, S), Y", address)
            }
            DirectPageLongIndirect => {
                format!(" [${:02X}]", address)
            }
            AddressLongIndirect => {
                format!(" [${:04X}]", address)
            }
            DirectPageLongIndirectYIndex => {
                format!(" [${:02X}], Y", address)
            }
            BlockMove => {
                format!(" ${:02X}, ${:02X}", self.operands[0], self.operands[1])
            }
        })
    }

    fn get_intermediate_address(&self, offset: usize, op_bytes: &[u8], resolve: bool) -> u32 {
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
                    todo!()
                };
                let operand = u16::from_le_bytes([op_bytes[0], op_bytes[1]]);
                (bank << 16) | (operand as u32)
            }
            AddressIndirect | AddressLongIndirect => u16::from_le_bytes([op_bytes[0], op_bytes[1]]) as u32,
            Long | LongXIndex => u32::from_le_bytes([op_bytes[0], op_bytes[1], op_bytes[3], 0]),
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
