pub struct SecondaryHeader {
    pub layer2_scroll:              u8,
    pub main_entrance_pos:          (u8, u8),
    pub layer3:                     u8,
    pub main_entrance_mario_action: u8,
    pub main_entrance_screen:       u8,
    pub midway_entrance_screen:     u8,
    pub fg_initial_pos:             u8,
    pub bg_initial_pos:             u8,
    pub no_yoshi_level:             bool,
    pub vertical_level:             bool,
}
