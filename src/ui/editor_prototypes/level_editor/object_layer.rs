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
    screen:    u8,
    midway:    bool,
    secondary: bool,
    id:        u16,
}

#[derive(Clone, Debug, Default)]
pub(super) struct EditableObjectLayer {
    pub objects: Vec<EditableObject>,
    pub exits:   Vec<EditableExit>,
}

impl EditableObject {
    pub fn from_raw(current_screen: u32, object: Object) -> Option<Self> {
        (!object.is_exit() && !object.is_screen_jump()).then(|| EditableObject {
            x:        object.x() as u32 + (current_screen * SCREEN_WIDTH),
            y:        object.y() as u32,
            id:       object.standard_object_number(),
            settings: object.settings(),
        })
    }

    pub fn to_raw(&self, new_screen: bool) -> Object {
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

    pub fn to_raw(&self) -> Object {
        Object(
            ((self.screen as u32 & 0x1F) << 24)
                | ((self.midway as u32) << 19)
                | ((self.secondary as u32) << 17)
                | (self.id as u32 & 0x01FF),
        )
    }
}

impl EditableObjectLayer {
    pub fn parse_from_ram(cpu: &mut Cpu) -> Option<Self> {
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
                let object = EditableObject::from_raw(current_screen as u32, raw_object)?;
                layer.objects.push(object);
            }
        }

        Some(layer)
    }

    pub fn write_to_ram(cpu: &mut Cpu) {
        //
    }
}
