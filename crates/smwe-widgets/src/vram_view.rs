use std::sync::{Arc, Mutex};

use egui::{vec2, PaintCallback, Response, Sense, Ui, Vec2, Widget};
use egui_glow::CallbackFn;
use glow::Context;
use itertools::Itertools;
use smwe_render::{
    gfx_buffers::GfxBuffers,
    tile_renderer::{Tile, TileRenderer},
};

#[derive(Debug)]
pub struct VramView {
    renderer: Arc<Mutex<TileRenderer>>,
    gfx_bufs: GfxBuffers,
}

impl VramView {
    pub fn new(renderer: Arc<Mutex<TileRenderer>>, gfx_bufs: GfxBuffers) -> Self {
        Self { renderer, gfx_bufs }
    }

    pub fn new_renderer(gl: &Context) -> TileRenderer {
        let tiles = (0..0x10000 / 32)
            .map(|t| {
                let pos_x = (t % 16) * 8;
                let pos_y = (t / 16) * 8;
                let tile = t & 0x3FF;
                let scale = 8;
                let pal = (t >> 10) & 0x7;
                let params = scale | (pal << 8) | (t & 0xC000);
                Tile([pos_x, pos_y, tile, params])
            })
            .collect_vec();
        let mut renderer = TileRenderer::new(gl);
        renderer.set_tiles(gl, tiles);
        renderer
    }
}

impl Widget for VramView {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self { renderer, gfx_bufs } = self;
        let rect_size = vec2(16. * 8., 32. * 8.);
        let (rect, response) = ui.allocate_exact_size(rect_size, Sense::focusable_noninteractive());
        let screen_size = rect.size() * ui.ctx().pixels_per_point();
        ui.painter().add(PaintCallback {
            rect,
            callback: Arc::new(CallbackFn::new(move |_info, painter| {
                renderer.lock().expect("Cannot lock mutex on VRAM renderer").paint(
                    painter.gl(),
                    gfx_bufs,
                    screen_size,
                    Vec2::ZERO,
                );
            })),
        });
        response
    }
}
