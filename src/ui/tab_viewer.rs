use egui::{Ui, WidgetText};
use egui_dock::TabViewer;

use crate::ui::{
    tool::{DockableEditorTool, DockableEditorToolEnum},
    EditorState,
};

pub struct EditorToolTabViewer<'v> {
    pub state: &'v mut EditorState,
}

impl TabViewer for EditorToolTabViewer<'_> {
    type Tab = DockableEditorToolEnum;

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        tab.update(ui, self.state);
    }

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.title()
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        log::info!("Closed {}", tab.title().text());
        true
    }
}
