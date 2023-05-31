use std::sync::{Arc, Mutex};

use egui::{PaintCallback, Response, Sense, Ui, Vec2, Widget};
use egui_glow::CallbackFn;
use glow::Buffer;
use smwe_render::palette_renderer::{PaletteRenderer, PaletteUniforms};

#[derive(Debug)]
pub struct PaletteView<'s> {
    renderer:    Arc<Mutex<PaletteRenderer>>,
    palette_buf: Buffer,
    viewed_rows: ViewedPaletteRows,
    selection:   Option<SelectionType<'s>>,
    size:        Vec2,
}

#[derive(Copy, Clone, Debug)]
pub enum ViewedPaletteRows {
    All,
    BackgroundOnly,
    SpritesOnly,
}

#[derive(Copy, Clone, Debug)]
pub enum SelectionType<'s> {
    Row(&'s u32),
    Cell(&'s (u32, u32)),
}

impl<'s> PaletteView<'s> {
    pub fn new(renderer: Arc<Mutex<PaletteRenderer>>, palette_buf: Buffer, size: Vec2) -> Self {
        Self { renderer, palette_buf, viewed_rows: ViewedPaletteRows::All, selection: None, size }
    }

    pub fn viewed_rows(mut self, viewed_rows: ViewedPaletteRows) -> Self {
        self.viewed_rows = viewed_rows;
        self
    }

    pub fn selection(mut self, selection: SelectionType<'s>) -> Self {
        self.selection = Some(selection);
        self
    }
}

impl Widget for PaletteView<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let palette_renderer = Arc::clone(&self.renderer);
        let uniforms = PaletteUniforms { palette_buf: self.palette_buf };
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::click());
        ui.painter().add(PaintCallback {
            rect,
            callback: Arc::new(CallbackFn::new(move |_info, painter| {
                palette_renderer.lock().expect("Cannot lock mutex on palette renderer").paint(painter.gl(), &uniforms);
            })),
        });
        response
    }
}
