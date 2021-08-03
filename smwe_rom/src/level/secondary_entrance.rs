use crate::{
    error::RomError,
    snes_utils::{addr::AddrSnes, rom::Rom, rom_slice::SnesSlice},
};

pub const SECONDARY_ENTRANCE_TABLE: SnesSlice = SnesSlice::new(AddrSnes(0x05F800), 512);

pub struct SecondaryEntrance([u8; 4]);

impl SecondaryEntrance {
    pub fn read_from_rom(rom: &Rom, entrance_id: usize) -> Result<Self, RomError> {
        let take_table =
            |table_num| rom.view().slice_lorom(SECONDARY_ENTRANCE_TABLE.skip_forward(table_num))?.as_bytes();

        let bytes = [
            take_table(0)?[entrance_id],
            take_table(1)?[entrance_id],
            take_table(2)?[entrance_id],
            take_table(3)?[entrance_id],
        ];

        Ok(Self(bytes))
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
