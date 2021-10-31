use std::convert::TryInto;

use nom::{
    bytes::complete::{tag, take},
    combinator::cond,
    multi::many_till,
    IResult,
};

pub const NON_EXIT_INSTANCE_SIZE: usize = 3;
pub const EXIT_INSTANCE_SIZE: usize = 4;

pub type StandardObjectID = u8;
pub type ExtendedObjectID = u8;

#[derive(Clone)]
pub struct ExitInstance([u8; EXIT_INSTANCE_SIZE]);

#[derive(Clone)]
pub struct NonExitInstance([u8; NON_EXIT_INSTANCE_SIZE]);

#[derive(Clone)]
pub struct ScreenJumpInstance([u8; NON_EXIT_INSTANCE_SIZE]);

#[derive(Clone)]
pub enum ObjectInstance {
    Exit(ExitInstance),
    NonExit(NonExitInstance),
    ScreenJump(ScreenJumpInstance),
}

#[derive(Clone)]
pub struct ObjectLayer {
    _objects: Vec<ObjectInstance>,
}

impl ExitInstance {
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

impl NonExitInstance {
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

impl ScreenJumpInstance {
    pub fn screen_number(&self) -> u8 {
        // ---HHHHH -------- --------
        // screen_number = HHHHH
        self.0[0] & 0b11111
    }
}

impl ObjectInstance {
    pub fn is_extended(&self) -> bool {
        match self {
            ObjectInstance::NonExit(i) => i.is_extended(),
            ObjectInstance::Exit(_) | ObjectInstance::ScreenJump(_) => true,
        }
    }
}

impl ObjectLayer {
    fn parse_object(input: &[u8]) -> IResult<&[u8], ObjectInstance> {
        let (input, first_three) = take(3usize)(input)?;
        let (input, last) = cond((first_three[0] & 0b11100000) == 0 && first_three[2] == 0, take(1usize))(input)?;
        match last {
            Some(l) => {
                let instance = ExitInstance([first_three, l].concat().try_into().unwrap());
                Ok((input, ObjectInstance::Exit(instance)))
            }
            None => {
                if first_three[2] == 1 {
                    let instance = ScreenJumpInstance(first_three.try_into().unwrap());
                    Ok((input, ObjectInstance::ScreenJump(instance)))
                } else {
                    let instance = NonExitInstance(first_three.try_into().unwrap());
                    Ok((input, ObjectInstance::NonExit(instance)))
                }
            }
        }
    }

    #[rustfmt::skip]
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (objects, _)) = many_till(Self::parse_object, tag(&[0xFFu8]))(input)?;
        Ok((input, Self { _objects: objects }))
    }
}
