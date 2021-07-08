use std::convert::{TryFrom, TryInto};

use nom::{count, number::complete::le_u8, preceded, take, IResult};

use crate::addr::{AddrPc, AddrSnes};

pub const SECONDARY_ENTRANCE_TABLE_ADDR: AddrSnes = AddrSnes(0x05F800);
pub const SECONDARY_ENTRANCE_TABLE_SIZE: usize = 512;

pub struct SecondaryEntrance([u8; 4]);

impl SecondaryEntrance {
    pub fn read_from_rom(rom_data: &[u8], entrance_id: usize) -> IResult<&[u8], Self> {
        let bytes_before_table = SECONDARY_ENTRANCE_TABLE_SIZE - entrance_id;
        let pc_addr = AddrPc::try_from(SECONDARY_ENTRANCE_TABLE_ADDR).unwrap();
        let (input, bytes) = preceded!(
            rom_data,
            take!(pc_addr.0 - bytes_before_table),
            count!(preceded!(take!(SECONDARY_ENTRANCE_TABLE_SIZE), le_u8), 4)
        )?;
        Ok((input, Self(bytes.try_into().unwrap())))
    }

    pub fn destination_level(&self) -> u16 {
        // dddddddd -------- -------- ----D---
        // destination_level = Ddddddddd

        ((self.0[3] as u16 & 0b111) << 5) | self.0[0] as u16
    }

    pub fn bg_initial_pos(&self) -> u8 {
        // -------- bb------ -------- --------
        // by_initial_pos = bb

        self.0[1] >> 6
    }

    pub fn fg_initial_pos(&self) -> u8 {
        // -------- --ff---- -------- --------
        // fg_initial_pos = ff

        self.0[1] >> 4
    }

    pub fn entrance_xy_pos(&self) -> (u8, u8) {
        // -------- ----yyyy xxx----- --------
        // entrance_xy_pos = (xxx, yyyy)

        let x = self.0[2] >> 5;
        let y = self.0[1] & 0b1111;
        (x, y)
    }

    pub fn screen_number(&self) -> u8 {
        // -------- -------- ---SSSSS --------
        // screen_number = SSSSS

        self.0[2] & 0b11111
    }
}
