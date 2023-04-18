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
    pub fn from_tuple(
        (upper_left, lower_left, upper_right, lower_right): (Tile8x8, Tile8x8, Tile8x8, Tile8x8),
    ) -> Self {
        Self { upper_left, lower_left, upper_right, lower_right }
    }
}
