use smwe_emu::Cpu;
use smwe_rom::level::PrimaryHeader;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct LevelProperties {
    // Primary header
    pub palette_bg:      u8,
    pub level_length:    u8,
    pub back_area_color: u8,
    pub level_mode:      u8,
    pub layer3_priority: bool,
    pub music:           u8,
    pub sprite_gfx:      u8,
    pub timer:           u8,
    pub palette_sprite:  u8,
    pub palette_fg:      u8,
    pub item_memory:     u8,
    pub vertical_scroll: u8,
    pub fg_bg_gfx:       u8,

    // Other
    pub is_vertical: bool,
}

impl LevelProperties {
    pub fn parse_from_ram(cpu: &mut Cpu) -> Self {
        let raw_header = PrimaryHeader::new(&cpu.mem.extram[..5]);
        let is_vertical = cpu.mem.load_u8(0x5B) & 1 != 0;
        Self {
            palette_bg: raw_header.palette_bg(),
            level_length: raw_header.level_length(),
            back_area_color: raw_header.back_area_color(),
            level_mode: raw_header.level_mode(),
            layer3_priority: raw_header.layer3_priority(),
            music: raw_header.music(),
            sprite_gfx: raw_header.sprite_gfx(),
            timer: raw_header.timer(),
            palette_sprite: raw_header.palette_sprite(),
            palette_fg: raw_header.palette_fg(),
            item_memory: raw_header.item_memory(),
            vertical_scroll: raw_header.vertical_scroll(),
            fg_bg_gfx: raw_header.fg_bg_gfx(),
            is_vertical,
        }
    }

    pub fn write_to_ram(&self, cpu: &mut Cpu) {
        // Primary header
        cpu.mem.extram[0] = ((self.palette_bg & 0x07) << 5) | (self.level_length & 0x1F);
        cpu.mem.extram[1] = ((self.back_area_color & 0x07) << 5) | (self.level_mode & 0x1F);
        cpu.mem.extram[2] = ((self.layer3_priority as u8) << 7) | ((self.music & 0x07) << 4) | (self.sprite_gfx & 0x0F);
        cpu.mem.extram[3] = ((self.timer & 0x02) << 6) | ((self.palette_sprite & 0x07) << 3) | (self.palette_fg & 0x07);
        cpu.mem.extram[4] =
            ((self.item_memory & 0x02) << 6) | ((self.vertical_scroll & 0x02) << 4) | (self.fg_bg_gfx & 0x0F);

        // Other
        let b = cpu.mem.load_u8(0x5B);
        cpu.mem.store_u8(0x5B, if self.is_vertical { b | 1 } else { b & !1 });
    }
}
