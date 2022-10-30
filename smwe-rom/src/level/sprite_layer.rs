use std::convert::TryInto;

use nom::{
    bytes::complete::{tag, take},
    multi::many_till,
    IResult,
};

pub const SPRITE_INSTANCE_SIZE: usize = 3;

pub type SpriteID = u8;

#[derive(Clone)]
pub struct SpriteInstance([u8; SPRITE_INSTANCE_SIZE]);

#[derive(Clone)]
pub struct SpriteLayer {
    _sprites: Vec<SpriteInstance>,
}

impl SpriteInstance {
    pub fn xy_pos(&self) -> (u8, u8) {
        // yyyy---Y XXXX---- --------
        // xy_pos = (XXXX, Yyyyy)
        let x = self.0[1] >> 4;
        let y = {
            let hi = (self.0[0] & 0b1) << 4;
            let lo = self.0[0] >> 4;
            hi | lo
        };
        (x, y)
    }

    pub fn extra_bits(&self) -> u8 {
        // ----EE-- -------- --------
        // extra_bits = EE
        (self.0[0] >> 2) & 0b11
    }

    pub fn screen_number(&self) -> u8 {
        // ------S- ----ssss --------
        // screen_number = Sssss
        let hi = (self.0[0] & 0b10) << 3;
        let lo = self.0[1] & 0b1111;
        hi | lo
    }

    pub fn sprite_id(&self) -> SpriteID {
        // -------- -------- NNNNNNNN
        // sprite_id = NNNNNNNN
        self.0[2]
    }
}

impl SpriteLayer {
    /// Returns self and the number of bytes consumed by parsing.
    pub fn parse(input: &[u8]) -> IResult<&[u8], (Self, usize)> {
        let mut read_sprite_layer = many_till(take(SPRITE_INSTANCE_SIZE), tag(&[0xFFu8]));
        let (rest, (sprites_raw, _)) = read_sprite_layer(input)?;
        let sprites = sprites_raw.into_iter().map(|spr| SpriteInstance(spr.try_into().unwrap())).collect();
        let bytes_consumed = input.len() - rest.len();
        Ok((rest, (Self { _sprites: sprites }, bytes_consumed)))
    }
}
