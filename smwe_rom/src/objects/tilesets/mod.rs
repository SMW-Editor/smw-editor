mod data;

pub use data::{
    TILES_000_072,
    TILES_073_0FF,
    TILES_100_106,
    TILES_107_110,
    TILES_111_152,
    TILES_153_16D,
    TILES_16E_1C3,
    TILES_1C4_1C7,
    TILES_1C8_1EB,
    TILES_1EC_1EF,
    TILES_1F0_1FF,
};
use itertools::Itertools;
use nom::{combinator::map, multi::many0, number::complete::le_u16};

use crate::{
    error::TilesetParseError,
    objects::map16::{Map16Tile, Tile8x8},
    snes_utils::{rom::Rom, rom_slice::SnesSlice},
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
