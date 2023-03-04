use std::ops::RangeInclusive;

use crate::snes_utils::{
    addr::{AddrInner, AddrSnes},
    rom_slice::SnesSlice,
};

const fn map16_data_slice(addr: AddrInner, range: RangeInclusive<u32>) -> SnesSlice {
    const MAP16_TILE_SIZE: usize = 8;
    SnesSlice::new(AddrSnes(addr), (*range.end() - *range.start() + 1) as usize * MAP16_TILE_SIZE)
}

pub static TILES_000_072: SnesSlice = map16_data_slice(0x0D8000, 0x000..=0x072);

// Tileset-specific
pub static TILES_073_0FF: [SnesSlice; 5] = [
    map16_data_slice(0x0D8B70, 0x073..=0x0FF), // 0: Normal 1, 2; Cloud/Forest
    map16_data_slice(0x0DBC00, 0x073..=0x0FF), // 1: Castle 1
    map16_data_slice(0x0DC800, 0x073..=0x0FF), // 2: Rope 1, 2, 3
    map16_data_slice(0x0DD400, 0x073..=0x0FF), // 3: Underground 1, 2, 3; Switch Palace 2; Castle 2
    map16_data_slice(0x0DE300, 0x073..=0x0FF), // 4: Switch Palace 1; Ghost House 1, 2
];

// Tileset-specific
pub static TILES_100_106: [SnesSlice; 5] = [
    map16_data_slice(0x0D8398, 0x100..=0x106), // 0: Normal 1, 2; Cloud/Forest
    map16_data_slice(0x0DC068, 0x100..=0x106), // 1: Castle 1
    map16_data_slice(0x0DCC68, 0x100..=0x106), // 2: Rope 1, 2, 3
    map16_data_slice(0x0DD868, 0x100..=0x106), // 3: Underground 1, 2, 3; Switch Palace 2; Castle 2
    map16_data_slice(0x0DE768, 0x100..=0x106), // 4: Switch Palace 1; Ghost House 1, 2
];

pub static TILES_107_110: SnesSlice = map16_data_slice(0x0DC068, 0x107..=0x110);
pub static TILES_111_152: SnesSlice = map16_data_slice(0x0D83D0, 0x111..=0x152);

// Tileset-specific
pub static TILES_153_16D: [SnesSlice; 5] = [
    map16_data_slice(0x0D9028, 0x153..=0x16D), // 0: Normal 1, 2; Cloud/Forest
    map16_data_slice(0x0DC0B8, 0x153..=0x16D), // 1: Castle 1
    map16_data_slice(0x0DCCB8, 0x153..=0x16D), // 2: Rope 1, 2, 3
    map16_data_slice(0x0DD8B8, 0x153..=0x16D), // 3: Underground 1, 2, 3; Switch Palace 2; Castle 2
    map16_data_slice(0x0DE7B8, 0x153..=0x16D), // 4: Switch Palace 1; Ghost House 1, 2
];

pub static TILES_16E_1C3: SnesSlice = map16_data_slice(0x0D85E0, 0x16E..=0x1C3);
pub static TILES_1C4_1C7: SnesSlice = map16_data_slice(0x0D8890, 0x1C4..=0x1C7);
pub static TILES_1C8_1EB: SnesSlice = map16_data_slice(0x0D88B0, 0x1C8..=0x1EB);
pub static TILES_1EC_1EF: SnesSlice = map16_data_slice(0x0D89D0, 0x1EC..=0x1EF);
pub static TILES_1F0_1FF: SnesSlice = map16_data_slice(0x0D89F0, 0x1F0..=0x1FF);
