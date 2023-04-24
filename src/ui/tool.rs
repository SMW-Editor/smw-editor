use eframe::egui::Ui;
use egui::WidgetText;
use enum_dispatch::enum_dispatch;

use crate::ui::{
    dev_utils::{
        address_converter::UiAddressConverter,
        disassembler::UiDisassembler,
        gfx_viewer::UiGfxViewer,
        palette_viewer::UiPaletteViewer,
        rom_info::UiRomInfo,
        tiles16x16::UiTiles16x16,
        vram_view::UiVramView,
    },
    editor_prototypes::{block_editor::UiBlockEditor, code_editor::UiCodeEditor},
    EditorState,
};

#[enum_dispatch]
pub enum DockableEditorToolEnum {
    UiAddressConverter,
    UiBlockEditor,
    UiCodeEditor,
    UiDisassembler,
    UiGfxViewer,
    UiPaletteViewer,
    UiRomInfo,
    UiTiles16x16,
    UiVramView,
}

#[enum_dispatch(DockableEditorToolEnum)]
pub trait DockableEditorTool {
    fn update(&mut self, ui: &mut Ui, state: &mut EditorState);
    fn title(&self) -> WidgetText;
}
