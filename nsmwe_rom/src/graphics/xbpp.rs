pub enum XbppTile {
    Tile2bpp([u8; 2 * 8]),
    Tile4bpp([u8; 4 * 8]),
    Tile8bpp([u8; 8 * 8]),
}

impl XbppTile {
    pub fn get_color_index_at(&self, (row, col): (usize, usize)) -> usize {
        assert!(row < 8);
        assert!(col < 8);

        let index_mask = 1 << col as u8;
        match self {
            XbppTile::Tile2bpp(t) => {
                let bytes  = [
                    t[(2 * row) + 0x00] & index_mask,
                    t[(2 * row) + 0x01] & index_mask,
                ];
                ((bytes[0] << 0) | (bytes[1] << 1)) as usize
            }
            XbppTile::Tile4bpp(t) => {
                let bytes  = [
                    t[(2 * row) + 0x00] & index_mask,
                    t[(2 * row) + 0x01] & index_mask,
                    t[(2 * row) + 0x10] & index_mask,
                    t[(2 * row) + 0x11] & index_mask,
                ];
                ((bytes[0] << 0) | (bytes[1] << 1) |
                 (bytes[2] << 2) | (bytes[3] << 3)) as usize
            }
            XbppTile::Tile8bpp(t) => {
                let bytes  = [
                    t[(2 * row) + 0x00] & index_mask,
                    t[(2 * row) + 0x01] & index_mask,
                    t[(2 * row) + 0x10] & index_mask,
                    t[(2 * row) + 0x11] & index_mask,
                    t[(2 * row) + 0x20] & index_mask,
                    t[(2 * row) + 0x21] & index_mask,
                    t[(2 * row) + 0x30] & index_mask,
                    t[(2 * row) + 0x31] & index_mask,
                ];
                ((bytes[0] << 0) | (bytes[1] << 1) |
                 (bytes[2] << 2) | (bytes[3] << 3) |
                 (bytes[4] << 4) | (bytes[5] << 5) |
                 (bytes[6] << 6) | (bytes[7] << 7)) as usize
            }
        }
    }
}
