use std::convert::TryInto;

use nom::{many_till, tag, take, IResult};

pub const SPRITE_INSTANCE_SIZE: usize = 3;

pub type SpriteID = u8;

#[derive(Clone)]
pub struct SpriteInstance([u8; SPRITE_INSTANCE_SIZE]);

#[derive(Clone)]
pub struct SpriteLayer {
    sprites: Vec<SpriteInstance>,
}

impl SpriteInstance {
    pub fn xy_pos(&self) -> (u8, u8) {
        // yyyy---Y XXXX---- --------
        // xy_pos = (XXXX, Yyyyy)

        let x = (self.0[1] >> 4) & 0b1111;
        let y = {
            let hi = (self.0[0] & 0b1) << 4;
            let lo = (self.0[0] >> 4) & 0b1111;
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

        ((self.0[0] & 0b10) << 3) | (self.0[1] & 0b1111)
    }

    pub fn sprite_id(&self) -> SpriteID {
        // -------- -------- NNNNNNNN
        // sprite_id = NNNNNNNN

        self.0[2]
    }
}

impl SpriteLayer {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (sprites_raw, _)) = many_till!(input, take!(SPRITE_INSTANCE_SIZE), tag!(&[0xFFu8]))?;
        let sprites = sprites_raw.into_iter().map(|spr| SpriteInstance(spr.try_into().unwrap())).collect();
        Ok((input, Self { sprites }))
    }
}
