use crate::{
    graphics::gfx_file,
    objects::{gfx_list::ObjectGfxList, tilesets::TILESETS_COUNT},
    GfxFile,
};

// Format: YXPCCCTT TTTTTTTT
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Tile8x8(pub u16);

#[derive(Copy, Clone, Debug)]
pub struct Map16Tile {
    pub upper_left:  Tile8x8,
    pub lower_left:  Tile8x8,
    pub upper_right: Tile8x8,
    pub lower_right: Tile8x8,
}

pub type TileLayer = usize;

// -------------------------------------------------------------------------------------------------

impl Tile8x8 {
    pub fn tile_number(&self) -> u16 {
        // ------tt TTTTTTTT
        // tile_number = ttTTTTTTTT
        self.0 & 0x3FF
    }

    pub fn flip_y(&self) -> bool {
        // Y------- --------
        // flip_y = Y
        (self.0 >> 15) != 0
    }

    pub fn flip_x(&self) -> bool {
        // -X------ --------
        // flip_x = X
        ((self.0 >> 14) & 1) != 0
    }

    pub fn priority(&self) -> bool {
        // --P----- --------
        // priority = P
        ((self.0 >> 13) & 1) != 0
    }

    pub fn palette(&self) -> u8 {
        // ---CCC-- --------
        // palette = CCC
        ((self.0 >> 10) & 0b111) as u8
    }

    pub fn layer(&self) -> TileLayer {
        (self.tile_number() / 0x80) as TileLayer
    }
}

impl Map16Tile {
    pub fn gfx<'gfx>(
        &self, gfx_list: &ObjectGfxList, gfx_files: &'gfx [GfxFile], tileset: usize,
    ) -> [&'gfx gfx_file::Tile; 4] {
        assert!(tileset < TILESETS_COUNT);
        let ref_gfx = |tile| {
            let file_num = gfx_list.gfx_file_for_object_tile(tile, tileset);
            let tile_num = tile.tile_number() as usize % 0x80;
            &gfx_files[file_num].tiles[tile_num]
        };
        [ref_gfx(self.upper_left), ref_gfx(self.lower_left), ref_gfx(self.upper_right), ref_gfx(self.lower_right)]
    }
}
