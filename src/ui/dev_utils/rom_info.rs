use imgui::{ImString, Window};
use nsmwe_rom::RomInternalHeader;

use crate::{
    frame_context::FrameContext,
    ui::{title_with_id, UiTool, WindowId},
};

pub struct UiRomInfo {
    title:        ImString,
    display_data: Vec<ImString>,
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
                ImString::new(format!("Internal ROM name: {}", header.internal_rom_name)),
                ImString::new(format!("Map mode:          {}", header.map_mode)),
                ImString::new(format!("ROM type:          {}", header.rom_type)),
                ImString::new(format!("ROM size:          {} kB", header.rom_size_in_kb())),
                ImString::new(format!("SRAM size:         {} kB", header.sram_size_in_kb())),
                ImString::new(format!("Region:            {}", header.region_code)),
                ImString::new(format!("Developer ID:      ${:x}", header.developer_id)),
                ImString::new(format!("Version:           1.{}", header.version_number)),
            ],
        }
    }
}
