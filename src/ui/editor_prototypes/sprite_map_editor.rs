use egui::{Ui, WidgetText};
use smwe_render::tile_renderer::Tile;

use crate::ui::{tool::DockableEditorTool, EditorState};

pub struct UiSpriteMapEditor {
    tiles: Vec<Tile>,
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
