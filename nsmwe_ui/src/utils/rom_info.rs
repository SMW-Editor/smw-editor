use crate::UiTool;

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

        if !running {
            log::info!("Closed ROM Info");
        }
        running
    }
}

impl UiRomInfo {
    pub fn new(header: &RomInternalHeader) -> Self {
        log::info!("Opened ROM Info");
        UiRomInfo {
            display_data: vec![
                ImString::new(format!("Internal ROM name: {}",    header.internal_rom_name)),
                ImString::new(format!("Map mode:          {}",    header.map_mode)),
                ImString::new(format!("ROM type:          {}",    header.rom_type)),
                ImString::new(format!("ROM size:          {} kB", header.rom_size_in_kb())),
                ImString::new(format!("SRAM size:         {} kB", header.sram_size_in_kb())),
                ImString::new(format!("Region:            {}",    header.region_code)),
                ImString::new(format!("Developer ID:      ${:x}", header.developer_id)),
                ImString::new(format!("Version:           1.{}",  header.version_number)),
            ]
        }
    }
}
