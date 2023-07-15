use shrinkwraprs::Shrinkwrap;
use smwe_render::tile_renderer::Tile;

use crate::undo::Undo;

#[derive(Clone, Debug, Shrinkwrap)]
#[shrinkwrap(mutable)]
pub(super) struct SpriteTiles(pub Vec<Tile>);

impl Undo for SpriteTiles {
    fn from_bytes(bytes: Vec<u8>) -> Self {
        let tiles = bytes
            .chunks(16)
            .map(|chunk| chunk.try_into().expect("Length of bytes list not divisible by 16"))
            .map(Tile::from_le_bytes)
            .collect();
        Self(tiles)
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.0.iter().flat_map(|tile| tile.0.into_iter().flat_map(|x| x.to_le_bytes())).collect()
    }

    fn size_bytes(&self) -> usize {
        self.0.len() * std::mem::size_of::<Tile>()
    }
}
