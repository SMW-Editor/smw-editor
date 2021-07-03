use std::convert::TryInto;

use nom::{many_till, tag, take, IResult};

pub const OBJECT_INSTANCE_SIZE: usize = 3;

#[derive(Clone)]
pub struct ObjectInstance([u8; OBJECT_INSTANCE_SIZE]);

#[derive(Clone)]
pub struct ObjectLayer {
    objects: Vec<ObjectInstance>,
}

impl ObjectInstance {
    pub fn is_extended(&self) -> bool {
        self.std_obj_num() == 0
    }

    pub fn new_screen(&self) -> bool {
        ((self.0[0] >> 7) & 0b1) != 0
    }

    pub fn std_obj_num(&self) -> u8 {
        ((self.0[0] >> 5) & 0b11) | ((self.0[1] >> 4) & 0b1111)
    }

    pub fn xy_pos(&self) -> (u8, u8) {
        let x = (self.0[1] >> 0) & 0b1111;
        let y = (self.0[0] >> 0) & 0b11111;
        (x, y)
    }

    pub fn settings(&self) -> u8 {
        self.0[2]
    }
}

impl ObjectLayer {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (objects_raw, _)) = many_till!(input, take!(OBJECT_INSTANCE_SIZE), tag!(&[0xFFu8]))?;
        let objects = objects_raw.into_iter().map(|obj| ObjectInstance(obj.try_into().unwrap())).collect();
        Ok((input, Self { objects }))
    }
}
