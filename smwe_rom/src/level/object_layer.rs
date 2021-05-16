use nom::{many_till, number::complete::le_u24, tag, IResult};

#[derive(Clone)]
pub struct ObjectData {
    new_screen:  bool,
    std_obj_num: u8,
    xy_pos:      (u8, u8),
    settings:    u8,
}

#[derive(Clone)]
pub struct ObjectLayer {
    objects: Vec<ObjectData>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum ObjectLayerType {
    Layer1,
    Layer2,
}

impl ObjectData {
    pub fn is_extended(&self) -> bool {
        self.std_obj_num == 0
    }
}

impl ObjectLayer {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (objects_raw, _)) = many_till!(input, le_u24, tag!(&[0xFFu8]))?;
        let objects = objects_raw
            .into_iter()
            .map(|obj| {
                let new_screen = ((obj >> 23) & 1) != 0;
                let std_obj_num = (((obj >> 17) & 0b110000) | ((obj >> 12) & 0b1111)) as u8;
                let xy_pos = (((obj >> 8) & 0b1111) as u8, ((obj >> 16) & 0b11111) as u8);
                let settings = (obj & 0xFF) as u8;
                ObjectData { new_screen, std_obj_num, xy_pos, settings }
            })
            .collect();
        Ok((input, ObjectLayer { objects }))
    }
}
