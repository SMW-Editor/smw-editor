mod data;

pub use data::*;
use itertools::Itertools;
use nom::{combinator::map, multi::many0, number::complete::le_u16};
use thiserror::Error;

use crate::{
    objects::{
        animated_tile_data::AnimatedTileDataParseError,
        map16::{Map16Tile, Tile8x8},
    },
    snes_utils::rom_slice::SnesSlice,
    DataBlock,
    DataKind,
    RomDisassembly,
};

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Error)]
#[error("Could not parse Map16 tiles at:\n- {0}")]
pub enum TilesetParseError {
    Slice(SnesSlice),
    AnimatedTileData(AnimatedTileDataParseError),
}

// -------------------------------------------------------------------------------------------------

pub const TILESETS_COUNT: usize = 5;

// -------------------------------------------------------------------------------------------------

pub struct Tilesets {
    pub tiles: Vec<Tile>,
}

pub enum Tile {
    Shared(Map16Tile),
    TilesetSpecific([Map16Tile; TILESETS_COUNT]),
}

// -------------------------------------------------------------------------------------------------

impl Tilesets {
    pub fn parse(disasm: &mut RomDisassembly) -> Result<Self, TilesetParseError> {
        let mut parse_16x16 = |slice| {
            let it = disasm
                .rom_slice_at_block(DataBlock { slice, kind: DataKind::Tileset }, |_| TilesetParseError::Slice(slice))?
                .parse(many0(map(le_u16, Tile8x8)))?
                .into_iter()
                .tuples::<(Tile8x8, Tile8x8, Tile8x8, Tile8x8)>()
                .map(Map16Tile::from_tuple);
            Ok(it)
        };

        let mut tiles: Vec<Tile> = Vec::with_capacity(0x200);

        let tiles_000_072 = parse_16x16(TILES_000_072)?.map(Tile::Shared);
        let tiles_107_110 = parse_16x16(TILES_107_110)?.map(Tile::Shared);
        let tiles_111_152 = parse_16x16(TILES_111_152)?.map(Tile::Shared);
        let tiles_16e_1c3 = parse_16x16(TILES_16E_1C3)?.map(Tile::Shared);
        let tiles_1c4_1c7 = parse_16x16(TILES_1C4_1C7)?.map(Tile::Shared);
        let tiles_1c8_1eb = parse_16x16(TILES_1C8_1EB)?.map(Tile::Shared);
        let tiles_1ec_1ef = parse_16x16(TILES_1EC_1EF)?.map(Tile::Shared);
        let tiles_1f0_1ff = parse_16x16(TILES_1F0_1FF)?.map(Tile::Shared);

        let mut parse_tileset_specific = |slices: [SnesSlice; 5]| {
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

        let tiles_073_0ff = parse_tileset_specific(TILES_073_0FF)?;
        let tiles_100_106 = parse_tileset_specific(TILES_100_106)?;
        let tiles_153_16d = parse_tileset_specific(TILES_153_16D)?;

        tiles.extend(
            tiles_000_072
                .chain(tiles_073_0ff)
                .chain(tiles_100_106)
                .chain(tiles_107_110)
                .chain(tiles_111_152)
                .chain(tiles_153_16d)
                .chain(tiles_16e_1c3)
                .chain(tiles_1c4_1c7)
                .chain(tiles_1c8_1eb)
                .chain(tiles_1ec_1ef)
                .chain(tiles_1f0_1ff),
        );

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
