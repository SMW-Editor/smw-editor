use egui::{Ui, WidgetText};

use crate::ui::{tool::DockableEditorTool, EditorState};

pub struct UiSpriteTileEditor {
    tiles: Vec<[u32; 4]>,
}

impl Default for UiSpriteTileEditor {
    fn default() -> Self {
        Self { tiles: Vec::new() }
    }
}

impl DockableEditorTool for UiSpriteTileEditor {
    fn update(&mut self, ui: &mut Ui, state: &mut EditorState) {
        //
    }

    fn title(&self) -> WidgetText {
        "Sprite Tile Editor".into()
    }
}
