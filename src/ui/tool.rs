#![allow(clippy::enum_variant_names)]

use eframe::egui::Ui;
use egui::WidgetText;
use enum_dispatch::enum_dispatch;

use crate::ui::{
    dev_utils::address_converter::UiAddressConverter,
    editor_prototypes::{
        block_editor::UiBlockEditor,
        level_editor::UiLevelEditor,
        sprite_map_editor::UiSpriteMapEditor,
    },
};

#[enum_dispatch]
pub enum DockableEditorToolEnum {
    UiAddressConverter,
    UiBlockEditor,
    UiLevelEditor,
    UiSpriteMapEditor,
}

#[enum_dispatch(DockableEditorToolEnum)]
pub trait DockableEditorTool {
    fn update(&mut self, ui: &mut Ui);
    fn title(&self) -> WidgetText;
    fn on_closed(&mut self) {}
}
