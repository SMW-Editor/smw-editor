#![allow(clippy::identity_op)]
#![allow(dead_code)]

use smwe_emu::Cpu;
use smwe_rom::objects::Object;

const SCREEN_WIDTH: u32 = 16;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct EditableObject {
    pub x:        u32,
    pub y:        u32,
    pub id:       u8,
    pub settings: u8,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct EditableExit {
    pub screen:    u8,
    pub midway:    bool,
    pub secondary: bool,
    pub id:        u16,
}

#[derive(Clone, Debug, Default)]
pub(super) struct EditableObjectLayer {
    pub objects: Vec<EditableObject>,
    pub exits:   Vec<EditableExit>,
}

impl EditableObject {
    pub fn from_raw(object: Object, current_screen: u32, vertical_level: bool) -> Option<Self> {
        (!object.is_exit() && !object.is_screen_jump()).then(|| EditableObject {
            x:        object.x() as u32 + if vertical_level { 0 } else { current_screen * SCREEN_WIDTH },
            y:        object.y() as u32 + if vertical_level { current_screen * SCREEN_WIDTH } else { 0 },
            id:       object.standard_object_number(),
            settings: object.settings(),
        })
    }

    pub fn to_raw(self, new_screen: bool) -> Object {
        Object(
            ((new_screen as u32) << 31)
                | ((self.id as u32 & 0x30) << 30)
                | ((self.id as u32 & 0x0F) << 20)
                | ((self.y & 0x1F) << 24)
                | ((self.x & 0x0F) << 16)
                | (self.settings as u32),
        )
    }
}

impl EditableExit {
    pub fn from_raw(object: Object) -> Option<Self> {
        object.is_exit().then(|| EditableExit {
            screen:    object.screen_number(),
            midway:    object.is_midway(),
            secondary: object.is_secondary_exit(),
            id:        object.exit_id(),
        })
    }

    pub fn to_raw(self) -> Object {
        Object(
            ((self.screen as u32 & 0x1F) << 24)
                | ((self.midway as u32) << 19)
                | ((self.secondary as u32) << 17)
                | (self.id as u32 & 0x01FF),
        )
    }
}

impl EditableObjectLayer {
    pub fn parse_from_ram(cpu: &mut Cpu, is_vertical_level: bool) -> Option<Self> {
        let raw_objects = Object::parse_from_ram(&cpu.mem.extram)?;
        let mut layer = Self::default();
        let mut current_screen = 0;

        for raw_object in raw_objects.into_iter() {
            if raw_object.is_exit() {
                layer.exits.push(EditableExit::from_raw(raw_object)?);
            } else if raw_object.is_screen_jump() {
                current_screen = raw_object.screen_number();
            } else {
                if raw_object.is_new_screen() {
                    current_screen += 1;
                }
                let object = EditableObject::from_raw(raw_object, current_screen as u32, is_vertical_level)?;
                layer.objects.push(object);
            }
        }

        Some(layer)
    }

    pub fn write_to_extram(&mut self, cpu: &mut Cpu, is_vertical_level: bool) {
        let mut current_offset = 5; // Start after primary header

        // Exits
        for exit in self.exits.iter() {
            Self::store_object_at(cpu, exit.to_raw(), &mut current_offset, 4);
        }

        if is_vertical_level {
            self.objects.sort_by(|a, b| a.y.cmp(&b.y));
        } else {
            self.objects.sort_by(|a, b| a.x.cmp(&b.x));
        }

        let mut current_screen = 0;
        for object in self.objects.iter() {
            let screen_number = if is_vertical_level { object.y } else { object.x } / SCREEN_WIDTH;
            debug_assert!(screen_number >= current_screen, "screen numbers must be in increasing order");
            debug_assert!(screen_number < 0x20, "exceeded maximum number of screens");

            let screen_difference = screen_number - current_screen;
            current_screen = screen_number;

            let new_screen = match (screen_difference > 0, screen_difference > 1) {
                (true, true) => {
                    // Screen jump
                    let jump = Object(((screen_number & 0x1F) << 16) | 0x100);
                    Self::store_object_at(cpu, jump, &mut current_offset, 3);
                    false
                }
                (new_screen, _) => new_screen,
            };

            // Standard/extended object
            Self::store_object_at(cpu, object.to_raw(new_screen), &mut current_offset, 3);
        }
    }

    fn store_object_at(cpu: &mut Cpu, object: Object, current_offset: &mut usize, length: usize) {
        debug_assert!(length > 0);
        debug_assert!(length <= 4);
        for byte_index in 0..length {
            cpu.mem.extram[*current_offset] = (object.0 >> ((3 - byte_index) * 8)) as u8;
            *current_offset += 1;
        }
    }
}
