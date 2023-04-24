use egui::{Ui, WidgetText};
use smwe_project::ProjectRef;

use crate::ui::tool::DockableEditorTool;

pub struct UiVramView {
    //
}

impl Default for UiVramView {
    fn default() -> Self {
        Self {}
    }
}

impl DockableEditorTool for UiVramView {
    fn update(&mut self, ui: &mut Ui, project_ref: &mut Option<ProjectRef>) {
        //
    }

    fn title(&self) -> WidgetText {
        "VRAM View".into()
    }
}
