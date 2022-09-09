use std::convert::TryInto;

use nom::{
    bytes::complete::{tag, take},
    multi::many_till,
    IResult,
};

pub const NON_EXIT_INSTANCE_SIZE: usize = 3;
pub const EXIT_INSTANCE_SIZE: usize = 4;

pub type StandardObjectID = u8;
pub type ExtendedObjectID = u8;

#[derive(Clone)]
pub struct StandardObject([u8; NON_EXIT_INSTANCE_SIZE]);

#[derive(Clone)]
pub struct ExitObject([u8; EXIT_INSTANCE_SIZE]);

#[derive(Clone)]
pub struct ScreenJumpObject([u8; NON_EXIT_INSTANCE_SIZE]);

#[derive(Clone)]
pub struct ExtendedOtherObject([u8; NON_EXIT_INSTANCE_SIZE]);

#[derive(Clone)]
pub enum ExtendedInstance {
    Exit(ExitObject),
    ScreenJump(ScreenJumpObject),
    Other(ExtendedOtherObject),
}

#[derive(Clone)]
pub enum ObjectInstance {
    Standard(StandardObject),
    Extended(ExtendedInstance),
}

#[derive(Clone)]
pub struct ObjectLayer {
    _objects: Vec<ObjectInstance>,
}

impl ExitObject {
    pub fn screen_number(&self) -> u8 {
        // ---ppppp -------- -------- --------
        // screen_number = ppppp
        self.0[0] & 0b11111
    }

    pub fn secondary_exit(&self) -> bool {
        // -------- ------s- -------- --------
        // secondary_exit = s
        (self.0[1] & 0b10) != 0
    }

    pub fn destination_level(&self) -> u16 {
        // -------- -------D -------- dddddddd
        // destination_level = Ddddddddd
        let hi = (self.0[1] as u16 & 0b1) << 8;
        let lo = self.0[3] as u16;
        hi | lo
    }
}

impl StandardObject {
    pub fn new_screen(&self) -> bool {
        // N----- -------- --------
        // new_screen = N
        (self.0[0] >> 7) != 0
    }

    pub fn std_obj_num(&self) -> StandardObjectID {
        // -BB----- bbbb---- --------
        // std_obj_num = BBbbbb
        let hi = (self.0[0] >> 1) & 0b110000;
        let lo = (self.0[1] >> 4) & 0b1111;
        hi | lo
    }

    pub fn ext_obj_num(&self) -> Option<ExtendedObjectID> {
        if self.is_extended() {
            // -------- -------- NNNNNNNN
            // ext_obj_num = NNNNNNNN
            Some(self.0[2])
        } else {
            None
        }
    }

    pub fn xy_pos(&self) -> (u8, u8) {
        // ---YYYYY ----XXXX --------
        // xy_pos = (XXXX, YYYYY)
        let x = self.0[1] & 0b1111;
        let y = self.0[0] & 0b11111;
        (x, y)
    }

    pub fn settings(&self) -> u8 {
        // -------- -------- SSSSSSSS
        // settings = SSSSSSSS
        self.0[2]
    }

    pub fn is_extended(&self) -> bool {
        self.std_obj_num() == 0
    }
}

impl ScreenJumpObject {
    pub fn screen_number(&self) -> u8 {
        // ---HHHHH -------- --------
        // screen_number = HHHHH
        self.0[0] & 0b11111
    }
}

impl ObjectInstance {
    pub fn is_extended(&self) -> bool {
        match self {
            ObjectInstance::Standard(i) => i.is_extended(),
            ObjectInstance::Extended(_) => true,
        }
    }
}

impl ObjectLayer {
    fn parse_object(input: &[u8]) -> IResult<&[u8], ObjectInstance> {
        let (input, first_three) = take(3usize)(input)?;
        if first_three[0] & 0b01100000 == 0 && first_three[1] & 0b11110000 == 0 {
            // Extended objects
            match first_three[2] {
                0 => {
                    let (input, last) = take(1usize)(input)?;
                    let instance = ExtendedInstance::Exit(ExitObject([first_three, last].concat().try_into().unwrap()));
                    Ok((input, ObjectInstance::Extended(instance)))
                }
                1 => {
                    let instance = ExtendedInstance::ScreenJump(ScreenJumpObject(first_three.try_into().unwrap()));
                    Ok((input, ObjectInstance::Extended(instance)))
                }
                _ => {
                    let instance = ExtendedInstance::Other(ExtendedOtherObject(first_three.try_into().unwrap()));
                    Ok((input, ObjectInstance::Extended(instance)))
                }
            }
        } else {
            // Standard objects
            let instance = StandardObject(first_three.try_into().unwrap());
            Ok((input, ObjectInstance::Standard(instance)))
        }
    }

    /// Returns self and the number of bytes consumed by parsing.
    pub fn parse(input: &[u8]) -> IResult<&[u8], (Self, usize)> {
        let (rest, (objects, _)) = many_till(Self::parse_object, tag(&[0xFFu8]))(input)?;
        let bytes_consumed = input.len() - rest.len();
        Ok((rest, (Self { _objects: objects }, bytes_consumed)))
    }
}
