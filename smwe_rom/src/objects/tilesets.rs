use std::ops::RangeInclusive;

use itertools::Itertools;
use nom::{combinator::map, multi::many0, number::complete::le_u16};

use crate::{
    error::TilesetParseError,
    objects::map16::{Map16Tile, Tile8x8},
    snes_utils::{addr::AddrSnes, rom::Rom, rom_slice::SnesSlice},
};

// -------------------------------------------------------------------------------------------------

pub struct Tilesets {
    tiles: Vec<Tile>,
}

pub enum Tile {
    Shared(Map16Tile),
    TilesetSpecific([Map16Tile; 5]),
}

// -------------------------------------------------------------------------------------------------

impl Tilesets {
    pub fn parse(rom: &Rom) -> Result<Self, TilesetParseError> {
        let parse_16x16 = |slice| {
            let it = rom
                .with_error_mapper(|_| TilesetParseError(slice))
                .slice_lorom(slice)?
                .parse(many0(map(le_u16, Tile8x8)))?
                .into_iter()
                .tuple_windows::<(Tile8x8, Tile8x8, Tile8x8, Tile8x8)>()
                .map(|(upper_left, lower_left, upper_right, lower_right)| Map16Tile {
                    upper_left,
                    lower_left,
                    upper_right,
                    lower_right,
                });
            Ok(it)
        };

        let parse_shared = |slice| Ok(parse_16x16(slice)?.map(Tile::Shared));

        let parse_tileset_specific = |slices: [SnesSlice; 5]| {
            let it = itertools::izip!(
                parse_16x16(slices[0])?,
                parse_16x16(slices[1])?,
                parse_16x16(slices[2])?,
                parse_16x16(slices[3])?,
                parse_16x16(slices[4])?,
            )
            .map(|(e0, e1, e2, e3, e4)| Tile::TilesetSpecific([e0, e1, e2, e3, e4]));
            Ok(it)
        };

        let mut tiles: Vec<Tile> = Vec::with_capacity(0x200);

        tiles.extend(parse_shared(TILES_000_072)?);
        tiles.extend(parse_tileset_specific(TILES_073_0FF)?);
        tiles.extend(parse_tileset_specific(TILES_100_106)?);
        tiles.extend(parse_shared(TILES_107_110)?);
        tiles.extend(parse_shared(TILES_111_152)?);
        tiles.extend(parse_tileset_specific(TILES_153_16D)?);
        tiles.extend(parse_shared(TILES_16E_1C3)?);
        tiles.extend(parse_shared(TILES_1C4_1C7)?);
        tiles.extend(parse_shared(TILES_1C8_1EB)?);
        tiles.extend(parse_shared(TILES_1EC_1EF)?);
        tiles.extend(parse_shared(TILES_1F0_1FF)?);

        Ok(Tilesets { tiles })
    }

    pub fn get_map16_tile(&self, tile_num: usize, tileset: usize) -> Option<Map16Tile> {
        if tile_num < self.tiles.len() && tileset < 5 {
            match self.tiles[tile_num] {
                Tile::Shared(tile) => Some(tile),
                Tile::TilesetSpecific(tiles) => Some(tiles[tileset]),
            }
        } else {
            log::error!("Invalid tile_num ({:#X}) or tileset ({})", tile_num, tileset);
            None
        }
    }
}

// -------------------------------------------------------------------------------------------------

const fn map16_data_slice(addr: usize, range: RangeInclusive<usize>) -> SnesSlice {
    const MAP16_TILE_SIZE: usize = 8;
    SnesSlice::new(AddrSnes(addr), (*range.end() - *range.start() + 1) * MAP16_TILE_SIZE)
}

static TILES_000_072: SnesSlice = map16_data_slice(0x0D8000, 0x000..=0x072);

// Tileset-specific
static TILES_073_0FF: [SnesSlice; 5] = [
    map16_data_slice(0x0D8B70, 0x073..=0x0FF), // 0: Normal 1, 2; Cloud/Forest
    map16_data_slice(0x0DBC00, 0x073..=0x0FF), // 1: Castle 1
    map16_data_slice(0x0DC800, 0x073..=0x0FF), // 2: Rope 1, 2, 3
    map16_data_slice(0x0DD400, 0x073..=0x0FF), // 3: Underground 1, 2, 3; Switch Palace 2; Castle 2
    map16_data_slice(0x0DE300, 0x073..=0x0FF), // 4: Switch Palace 1; Ghost House 1, 2
];

// Tileset-specific
static TILES_100_106: [SnesSlice; 5] = [
    map16_data_slice(0x0D8398, 0x100..=0x106), // 0: Normal 1, 2; Cloud/Forest
    map16_data_slice(0x0DC068, 0x100..=0x106), // 1: Castle 1
    map16_data_slice(0x0DCC68, 0x100..=0x106), // 2: Rope 1, 2, 3
    map16_data_slice(0x0DD868, 0x100..=0x106), // 3: Underground 1, 2, 3; Switch Palace 2; Castle 2
    map16_data_slice(0x0DE768, 0x100..=0x106), // 4: Switch Palace 1; Ghost House 1, 2
];

static TILES_107_110: SnesSlice = map16_data_slice(0x0DC068, 0x107..=0x110);
static TILES_111_152: SnesSlice = map16_data_slice(0x0D83D0, 0x111..=0x152);

// Tileset-specific
static TILES_153_16D: [SnesSlice; 5] = [
    map16_data_slice(0x0D9028, 0x153..=0x16D), // 0: Normal 1, 2; Cloud/Forest
    map16_data_slice(0x0DC0B8, 0x153..=0x16D), // 1: Castle 1
    map16_data_slice(0x0DCCB8, 0x153..=0x16D), // 2: Rope 1, 2, 3
    map16_data_slice(0x0DD8B8, 0x153..=0x16D), // 3: Underground 1, 2, 3; Switch Palace 2; Castle 2
    map16_data_slice(0x0DE7B8, 0x153..=0x16D), // 4: Switch Palace 1; Ghost House 1, 2
];

static TILES_16E_1C3: SnesSlice = map16_data_slice(0x0D85E0, 0x16E..=0x1C3);
static TILES_1C4_1C7: SnesSlice = map16_data_slice(0x0D8890, 0x1C4..=0x1C7);
static TILES_1C8_1EB: SnesSlice = map16_data_slice(0x0D88B0, 0x1C8..=0x1EB);
static TILES_1EC_1EF: SnesSlice = map16_data_slice(0x0D89D0, 0x1EC..=0x1EF);
static TILES_1F0_1FF: SnesSlice = map16_data_slice(0x0D89F0, 0x1F0..=0x1FF);
