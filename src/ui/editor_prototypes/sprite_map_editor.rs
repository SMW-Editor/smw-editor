use egui::{Ui, WidgetText};

use crate::ui::{tool::DockableEditorTool, EditorState};

pub struct UiSpriteMapEditor {
    tiles: Vec<[u32; 4]>,
}

impl Default for UiSpriteMapEditor {
    fn default() -> Self {
        Self { tiles: Vec::new() }
    }
}

impl DockableEditorTool for UiSpriteMapEditor {
    fn update(&mut self, ui: &mut Ui, state: &mut EditorState) {
        //
    }

    fn title(&self) -> WidgetText {
        "Sprite Tile Editor".into()
    }
}
