use std::sync::{Arc, Mutex};

use egui::{vec2, PaintCallback, Response, Sense, Ui, Vec2, Widget};
use egui_glow::CallbackFn;
use smwe_render::{gfx_buffers::GfxBuffers, tile_renderer::TileRenderer};

#[derive(Debug)]
pub struct VramView {
    renderer: Arc<Mutex<TileRenderer>>,
    gfx_bufs: GfxBuffers,
}

impl VramView {
    pub fn new(renderer: Arc<Mutex<TileRenderer>>, gfx_bufs: GfxBuffers) -> Self {
        Self { renderer, gfx_bufs }
    }
}

impl Widget for VramView {
    fn ui(self, ui: &mut Ui) -> Response {
        let rect_size = vec2(16. * 8., 32. * 8.);
        let (rect, response) = ui.allocate_exact_size(rect_size, Sense::focusable_noninteractive());
        ui.painter().add(PaintCallback {
            rect,
            callback: Arc::new(CallbackFn::new(move |_info, painter| {
                self.renderer.lock().expect("Cannot lock mutex on VRAM renderer").paint(
                    painter.gl(),
                    self.gfx_bufs,
                    rect_size,
                    Vec2::ZERO,
                );
            })),
        });
        response
    }
}
