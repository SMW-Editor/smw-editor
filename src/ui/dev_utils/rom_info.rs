use imgui::Window;
use smwe_rom::RomInternalHeader;

use crate::{
    frame_context::FrameContext,
    ui::{title_with_id, UiTool, WindowId},
};

pub struct UiRomInfo {
    title:        String,
    display_data: Vec<String>,
}

impl UiTool for UiRomInfo {
    fn tick(&mut self, ctx: &mut FrameContext) -> bool {
        let mut running = true;

        let title = std::mem::take(&mut self.title);
        Window::new(&title)
            .always_auto_resize(true)
            .resizable(false)
            .collapsible(false)
            .scroll_bar(false)
            .opened(&mut running)
            .build(ctx.ui, || {
                self.display_data.iter().for_each(|t| ctx.ui.text(t));
            });
        self.title = title;

        if !running {
            log::info!("Closed ROM Info");
        }
        running
    }
}

impl UiRomInfo {
    pub fn new(id: WindowId, header: &RomInternalHeader) -> Self {
        log::info!("Opened ROM Info");
        UiRomInfo {
            title:        title_with_id("ROM info", id),
            display_data: vec![
                format!("Internal ROM name: {}", header.internal_rom_name),
                format!("Map mode:          {}", header.map_mode),
                format!("ROM type:          {}", header.rom_type),
                format!("ROM size:          {} kB", header.rom_size_in_kb()),
                format!("SRAM size:         {} kB", header.sram_size_in_kb()),
                format!("Region:            {}", header.region_code),
                format!("Developer ID:      ${:x}", header.developer_id),
                format!("Version:           1.{}", header.version_number),
            ],
        }
    }
}
