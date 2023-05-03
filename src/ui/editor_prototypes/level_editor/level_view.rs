use std::sync::{Arc, Mutex};

use egui::{vec2, CentralPanel, Color32, Frame, PaintCallback, Sense, Ui, WidgetText};
use egui_glow::CallbackFn;
use smwe_rom::graphics::color::Abgr1555;

use crate::ui::{
    editor_prototypes::level_editor::level_renderer::LevelRenderer,
    tool::DockableEditorTool,
    EditorState,
};

pub struct UiLevelView {
    level_renderer: Arc<Mutex<LevelRenderer>>,
    offset:         [f32; 2],
}

impl UiLevelView {
    pub(super) fn new(level_renderer: Arc<Mutex<LevelRenderer>>) -> Self {
        Self { level_renderer, offset: [0., 0.] }
    }
}

impl DockableEditorTool for UiLevelView {
    fn update(&mut self, ui: &mut Ui, state: &mut EditorState) {
        let cpu = state.cpu.as_mut().unwrap();
        let bg_color = cpu.mem.load_u16(0x7E0701);
        let bg_color = Color32::from(Abgr1555(bg_color));
        CentralPanel::default().frame(Frame::none().inner_margin(0.).fill(bg_color)).show_inside(ui, |ui| {
            let level_renderer = Arc::clone(&self.level_renderer);
            let (rect, response) =
                ui.allocate_exact_size(vec2(ui.available_width(), ui.available_height()), Sense::click_and_drag());
            let screen_size = rect.size() * ui.ctx().pixels_per_point();

            if response.dragged() {
                let mut r = level_renderer.lock().unwrap();
                let delta = response.drag_delta();
                self.offset[0] += delta.x;
                self.offset[1] += delta.y;
                r.set_offsets(self.offset);
            }

            ui.painter().add(PaintCallback {
                rect,
                callback: Arc::new(CallbackFn::new(move |_info, painter| {
                    level_renderer
                        .lock()
                        .expect("Cannot lock mutex on level_renderer")
                        .paint(painter.gl(), screen_size);
                })),
            });
        });
    }

    fn title(&self) -> WidgetText {
        "Level View".into()
    }
}
