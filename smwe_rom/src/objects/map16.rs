use crate::snes_utils::{addr::AddrSnes, rom_slice::SnesSlice};
use std::ops::RangeInclusive;

// -------------------------------------------------------------------------------------------------

pub type BlockNum = u16;

// Format: TTTTTTTT YXPCCCTT
pub struct Tile8x8(u16);

pub struct Map16Tile {
    block_num: BlockNum,
    upper_left: Tile8x8,
    lower_left: Tile8x8,
    upper_right: Tile8x8,
    lower_right: Tile8x8,
}

// -------------------------------------------------------------------------------------------------

impl Tile8x8 {
    pub fn tile_number(&self) -> u16 {
        // TTTTTTTT ------tt
        // tile_number = TTTTTTTTtt
        (self.0 >> 6) | (self.0 & 0b11)
    }

    pub fn flip_y(&self) -> bool {
        // -------- Y-------
        // flip_y = Y
        (self.0 >> 7) != 0
    }

    pub fn flip_x(&self) -> bool {
        // -------- -X------
        // flip_x = X
        ((self.0 >> 6) & 1) != 0
    }

    pub fn priority(&self) -> bool {
        // -------- --P-----
        // priority = P
        ((self.0 >> 5) & 1) != 0
    }

    pub fn palette(&self) -> u8 {
        // -------- ---CCC--
        // palette = CCC
        ((self.0 >> 2) & 0b111) as u8
    }
}

// -------------------------------------------------------------------------------------------------

const fn map16_data_slice(addr: usize, range: RangeInclusive<usize>) -> SnesSlice {
    const MAP16_TILE_SIZE: usize = 8;
    SnesSlice::new(AddrSnes(addr), (*range.end() - *range.start() + 1) * MAP16_TILE_SIZE)
}

pub(crate) static TILES_000_072: SnesSlice = map16_data_slice(0x0D8000, 0x000..=0x072);

// Tileset-specific
pub(crate) static MAP16_TILES_073_0FF: [SnesSlice; 5] = [
    map16_data_slice(0x0D8B70, 0x073..=0x0FF), // 0: Normal 1, 2; Cloud/Forest
    map16_data_slice(0x0DBC00, 0x073..=0x0FF), // 1: Castle 1
    map16_data_slice(0x0DC800, 0x073..=0x0FF), // 2: Rope 1, 2, 3
    map16_data_slice(0x0DD400, 0x073..=0x0FF), // 3: Underground 1, 2, 3; Switch Palace 2; Castle 2
    map16_data_slice(0x0DE300, 0x073..=0x0FF), // 4: Switch Palace 1; Ghost House 1, 2
];

// Tileset-specific
pub(crate) static TILES_100_106: [SnesSlice; 5] = [
    map16_data_slice(0x0D8398, 0x100..=0x106), // 0: Normal 1, 2; Cloud/Forest
    map16_data_slice(0x0DC068, 0x100..=0x106), // 1: Castle 1
    map16_data_slice(0x0DCC68, 0x100..=0x106), // 2: Rope 1, 2, 3
    map16_data_slice(0x0DD868, 0x100..=0x106), // 3: Underground 1, 2, 3; Switch Palace 2; Castle 2
    map16_data_slice(0x0DE768, 0x100..=0x106), // 4: Switch Palace 1; Ghost House 1, 2
];

pub(crate) static TILES_107_110: SnesSlice = map16_data_slice(0x0DC068, 0x107..=0x110);
pub(crate) static TILES_111_152: SnesSlice = map16_data_slice(0x0D83D0, 0x111..=0x152);

// Tileset-specific
pub(crate) static TILES_153_16D: [SnesSlice; 5] = [
    map16_data_slice(0x0D9028, 0x153..=0x16D), // 0: Normal 1, 2; Cloud/Forest
    map16_data_slice(0x0DC0B8, 0x153..=0x16D), // 1: Castle 1
    map16_data_slice(0x0DCCB8, 0x153..=0x16D), // 2: Rope 1, 2, 3
    map16_data_slice(0x0DD8B8, 0x153..=0x16D), // 3: Underground 1, 2, 3; Switch Palace 2; Castle 2
    map16_data_slice(0x0DE7B8, 0x153..=0x16D), // 4: Switch Palace 1; Ghost House 1, 2
];

pub(crate) static TILES_16E_1C3: SnesSlice = map16_data_slice(0x0D85E0, 0x16E..=0x1C3);
pub(crate) static TILES_1C4_1C7: SnesSlice = map16_data_slice(0x0D8890, 0x1C4..=0x1C7);
pub(crate) static TILES_1C8_1EB: SnesSlice = map16_data_slice(0x0D88B0, 0x1C8..=0x1EB);
pub(crate) static TILES_1EC_1EF: SnesSlice = map16_data_slice(0x0D89D0, 0x1EC..=0x1EF);
pub(crate) static TILES_1F0_1FF: SnesSlice = map16_data_slice(0x0D89F0, 0x1F0..=0x1FF);
