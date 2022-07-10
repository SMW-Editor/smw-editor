use eframe::egui::{RichText, Ui, Window};
use smwe_rom::RomInternalHeader;
use crate::{
    frame_context::EFrameContext,
    ui_new::tool::UiTool,
};

pub struct UiRomInfo {
    display_data: Vec<(String, String)>,
}

impl UiTool for UiRomInfo {
    fn update(&mut self, ui: &mut Ui, ctx: &mut EFrameContext) -> bool {
        let mut running = true;

        Window::new("Internal ROM Header")
            .auto_sized()
            .collapsible(false)
            .open(&mut running)
            .show(ctx.ctx, |ui| {
                for (name, data) in self.display_data.iter() {
                    ui.horizontal(|ui| {
                        ui.label(name);
                        ui.label(RichText::new(data).monospace());
                    });
                }
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
        Self {
            display_data: vec![
                (String::from("Internal ROM name:"), header.internal_rom_name.clone()),
                (String::from("Map mode:"), format!("{}", header.map_mode)),
                (String::from("ROM type:"), format!("{}", header.rom_type)),
                (String::from("ROM size:"), format!("{} kB", header.rom_size_in_kb())),
                (String::from("SRAM size:"), format!("{} kB", header.sram_size_in_kb())),
                (String::from("Region:"), format!("{}", header.region_code)),
                (String::from("Developer ID:"), format!("0x{:x}", header.developer_id)),
                (String::from("Version:"), format!("1.{}", header.version_number)),
            ],
        }
    }
}
