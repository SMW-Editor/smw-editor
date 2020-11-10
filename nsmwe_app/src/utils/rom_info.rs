use crate::tool::UiTool;

use imgui::{
    Condition,
    ImString,
    Ui,
    Window,
    im_str,
};
use inline_tweak::tweak;
use std::str;

pub struct UiRomInfo {
    display_data: Vec<ImString>,
}

impl UiTool for UiRomInfo {
    fn run(&mut self, ui: &Ui) -> bool {
        let mut running = true;
        Window::new(im_str!("ROM info"))
            .size([tweak!(270.0), tweak!(150.0)], Condition::Always)
            .resizable(false)
            .collapsible(false)
            .scroll_bar(false)
            .opened(&mut running)
            .build(ui, || {
                self.display_data.iter().for_each(|t| ui.text(t));
            });
        running
    }
}

impl UiRomInfo {
    pub fn new() -> Self {
        use nsmwe_rom::parse_rom_data;

        let rom_data = std::fs::read(std::env::var("ROM_PATH")
            .expect("The path to the ROM file (variable ROM_PATH) is undefined."))
            .unwrap();
        let header = parse_rom_data(&rom_data).unwrap().internal_header;

        let internal_rom_name = str::from_utf8(&header.internal_rom_name)
            .unwrap_or("error");

        UiRomInfo {
            display_data: vec![
                ImString::from(format!("Internal ROM name: {}", internal_rom_name)),
                ImString::from(format!("Map mode: {}", header.map_mode)),
                ImString::from(format!("ROM type: {}", header.rom_type)),
                ImString::from(format!("ROM size: {} kB", header.rom_size_in_kb())),
                ImString::from(format!("SRAM size: {} kB", header.sram_size_in_kb())),
                ImString::from(format!("Region: {}", header.destination_code)),
                ImString::from(format!("Version: {:x}", header.version_number)),
            ]
        }
    }
}