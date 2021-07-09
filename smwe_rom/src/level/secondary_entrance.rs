use std::convert::{TryFrom, TryInto};

use nom::{bytes::complete::take, multi::count, number::complete::le_u8, sequence::preceded};

use crate::{
    addr::{AddrPc, AddrSnes},
    error::{ParseErr, SecondaryEntranceParseError},
};

pub const SECONDARY_ENTRANCE_TABLE_ADDR: AddrSnes = AddrSnes(0x05F800);
pub const SECONDARY_ENTRANCE_TABLE_SIZE: usize = 512;

pub struct SecondaryEntrance([u8; 4]);

impl SecondaryEntrance {
    pub fn read_from_rom(rom_data: &[u8], entrance_id: usize) -> Result<Self, SecondaryEntranceParseError> {
        let bytes_before_table = SECONDARY_ENTRANCE_TABLE_SIZE - entrance_id;
        let pc_addr: usize = AddrPc::try_from(SECONDARY_ENTRANCE_TABLE_ADDR)
            .map_err(|_| SecondaryEntranceParseError::TablesAddressConversion)?
            .into();
        let mut read_secondary_entrance = preceded(
            take(pc_addr - bytes_before_table),
            count(preceded(take(SECONDARY_ENTRANCE_TABLE_SIZE), le_u8), 4),
        );

        let (_, bytes) = read_secondary_entrance(rom_data).map_err(|_: ParseErr| SecondaryEntranceParseError::Read)?;

        Ok(Self(bytes.try_into().unwrap()))
    }

    pub fn destination_level(&self) -> u16 {
        // dddddddd -------- -------- ----D---
        // destination_level = Ddddddddd
        let hi = (self.0[3] as u16 & 0b1000) << 5;
        let lo = self.0[0] as u16;
        hi | lo
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
