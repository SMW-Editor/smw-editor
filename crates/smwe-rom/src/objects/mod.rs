pub mod animated_tile_data;
pub mod map16;
pub mod object_gfx_list;
pub mod tilesets;

/// # Object format
///
/// ## Standard
///
/// `NBBYYYYY bbbbXXXX SSSSSSSS`
///
/// | Value      | Comment                |
/// |------------|------------------------|
/// | `N`        | New Screen flag        |
/// | `BBbbbb`   | Standard object number |
/// | `YYYYY`    | Y position             |
/// | `XXXX`     | X position             |
/// | `SSSSSSSS` | Settings               |
///
/// ## Extended
///
/// `N00YYYYY 0000XXXX BBBBBBBB`
///
/// | Value    | Comment                |
/// |----------|------------------------|
/// | N        | New Screen flag        |
/// | YYYYY    | Y position             |
/// | XXXX     | X position             |
/// | BBBBBBBB | Extended object number |
///
/// ## Exit
///
/// `000ppppp 0000w0sh 00000000 dddddddd`
///
/// | Value     | Comment                               |
/// |-----------|---------------------------------------|
/// | ppppp     | Screen number                         |
/// | w         | Midway                                |
/// | s         | Secondary exit flag                   |
/// | hdddddddd | Destination level / secondary exit ID |
///
/// ## Screen jump
///
/// `000HHHHH 00000000 00000001`
///
/// | Value     | Comment       |
/// |-----------|---------------|
/// | HHHHH     | Screen number |
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Object(pub u32);

impl Object {
    pub fn parse_from_ram(buffer: &[u8]) -> Option<Vec<Self>> {
        if buffer.is_empty() {
            return None;
        }
        let mut objects = Vec::new();
        let mut it = &buffer[5..];
        loop {
            if it[0] == 0xFF {
                break;
            }
            if it[0] & 0x50 == 0 && it[1] & 0xF0 == 0 && it[2] == 0 {
                // Exits
                let [b1, b2, b3, b4] = it[..4] else {
                    return None;
                };
                let object = u32::from_be_bytes([b1, b2, b3, b4]);
                objects.push(Object(object));
                it = &it[4..];
            } else {
                // Non-exits
                let [b1, b2, b3] = it[..3] else {
                    return None;
                };
                let object = u32::from_be_bytes([b1, b2, b3, 0]);
                objects.push(Object(object));
                it = &it[3..];
            }
        }
        Some(objects)
    }
}

impl Object {
    #[inline(always)]
    pub fn is_extended(self) -> bool {
        // -00----- 0000---- -------- --------
        (self.0 >> 20) & 0x60F == 0
    }

    #[inline(always)]
    pub fn is_standard(self) -> bool {
        !self.is_extended()
    }

    #[inline(always)]
    pub fn is_exit(self) -> bool {
        self.is_extended() && self.settings() == 0
    }

    #[inline(always)]
    pub fn is_screen_jump(self) -> bool {
        self.is_extended() && self.settings() == 1
    }

    #[inline(always)]
    pub fn is_new_screen(self) -> bool {
        // N------- -------- -------- --------
        (self.0 >> 31) & 1 != 0
    }

    #[inline(always)]
    pub fn standard_object_number(self) -> u8 {
        // -BB----- bbbb---- -------- --------
        (((self.0 >> 25) & 0x30) | ((self.0 >> 20) & 0x0F)) as u8
    }

    #[inline(always)]
    pub fn settings(self) -> u8 {
        // -------- -------- SSSSSSSS --------
        (self.0 >> 8) as u8
    }

    #[inline(always)]
    pub fn y(self) -> u8 {
        debug_assert!(!self.is_exit() && !self.is_screen_jump());
        // ---YYYYY -------- -------- --------
        (self.0 >> 24) as u8 & 0x1F
    }

    #[inline(always)]
    pub fn x(self) -> u8 {
        debug_assert!(!self.is_exit() && !self.is_screen_jump());
        // -------- ----XXXX -------- --------
        (self.0 >> 16) as u8 & 0xF
    }

    #[inline(always)]
    pub fn xy(self) -> (u8, u8) {
        (self.x(), self.y())
    }

    #[inline(always)]
    pub fn screen_number(self) -> u8 {
        debug_assert!(self.is_exit() || self.is_screen_jump());
        // Exit:        ---ppppp -------- -------- --------
        // Screen junp: ---HHHHH -------- -------- --------
        (self.0 >> 24) as u8 & 0x1F
    }

    #[inline(always)]
    pub fn is_midway(self) -> bool {
        debug_assert!(self.is_exit());
        // -------- ----w--- -------- --------
        (self.0 >> 19) & 1 != 0
    }

    #[inline(always)]
    pub fn is_secondary_exit(self) -> bool {
        debug_assert!(self.is_exit());
        // -------- ------s- -------- --------
        (self.0 >> 17) & 1 != 0
    }

    #[inline(always)]
    pub fn exit_id(self) -> u16 {
        debug_assert!(self.is_exit());
        // -------- -------h -------- dddddddd
        ((self.0 & 0xFF) | ((self.0 >> 8) & 0x100)) as u16
    }
}
