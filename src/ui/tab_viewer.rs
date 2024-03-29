use egui::{Ui, WidgetText};
use egui_dock::TabViewer;
use smwe_project::ProjectRef;

use crate::ui::tool::{DockableEditorTool, DockableEditorToolEnum};

pub struct EditorToolTabViewer<'f> {
    pub project_ref: &'f mut Option<ProjectRef>,
}

impl TabViewer for EditorToolTabViewer<'_> {
    type Tab = DockableEditorToolEnum;

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        tab.update(ui, self.project_ref);
    }

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.title()
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        log::info!("Closed {}", tab.title().text());
        true
    }
}
