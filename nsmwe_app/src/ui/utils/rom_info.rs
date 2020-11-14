use crate::ui::UiTool;

use imgui::{
    ImString,
    Ui,
    Window,
    im_str,
};

use nsmwe_rom::RomInternalHeader;

pub struct UiRomInfo {
    display_data: Vec<ImString>,
}

impl UiTool for UiRomInfo {
    fn run(&mut self, ui: &Ui) -> bool {
        let mut running = true;

        Window::new(im_str!("ROM info"))
            .always_auto_resize(true)
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
    pub fn new(header: &RomInternalHeader) -> Self {
        UiRomInfo {
            display_data: vec![
                ImString::from(format!("Internal ROM name: {}",    header.internal_rom_name)),
                ImString::from(format!("Map mode:          {}",    header.map_mode)),
                ImString::from(format!("ROM type:          {}",    header.rom_type)),
                ImString::from(format!("ROM size:          {} kB", header.rom_size_in_kb())),
                ImString::from(format!("SRAM size:         {} kB", header.sram_size_in_kb())),
                ImString::from(format!("Region:            {}",    header.destination_code)),
                ImString::from(format!("Developer ID:      ${:x}", header.developer_id)),
                ImString::from(format!("Version:           1.{}",  header.version_number)),
            ]
        }
    }
}