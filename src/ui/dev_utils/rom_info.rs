use eframe::egui::{RichText, Ui};
use egui::WidgetText;
use egui_extras::{Column, TableBuilder};
use smwe_project::ProjectRef;
use smwe_rom::RomInternalHeader;

use crate::ui::tool::DockableEditorTool;

#[derive(Debug)]
pub struct UiRomInfo {
    display_data: Vec<(String, String)>,
}

impl DockableEditorTool for UiRomInfo {
    fn update(&mut self, ui: &mut Ui, _project_ref: &mut Option<ProjectRef>) {
        let min_scroll_height = ui.available_height();
        TableBuilder::new(ui) //
            .striped(true)
            .min_scrolled_height(min_scroll_height)
            .columns(Column::exact(130.0), 2)
            .body(|body| {
                body.rows(15.0, self.display_data.len(), |i, mut row| {
                    let (name, data) = &self.display_data[i];
                    row.col(|ui| {
                        ui.label(name);
                    });
                    row.col(|ui| {
                        ui.label(RichText::new(data).monospace());
                    });
                });
            });
    }

    fn title(&self) -> WidgetText {
        "Internal ROM Header".into()
    }
}

impl UiRomInfo {
    pub fn new(header: &RomInternalHeader) -> Self {
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
