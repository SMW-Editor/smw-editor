use std::sync::{Arc, Mutex};

use egui::{pos2, vec2, Color32, PaintCallback, Rect, Response, Rounding, Sense, Stroke, Ui, Vec2, Widget};
use egui_glow::CallbackFn;
use glow::Context;
use inline_tweak::tweak;
use itertools::Itertools;
use smwe_render::{
    gfx_buffers::GfxBuffers,
    tile_renderer::{Tile, TileRenderer},
};

#[derive(Copy, Clone, Debug)]
pub enum ViewedVramTiles {
    All,
    SpritesOnly,
    BackgroundOnly,
}

#[derive(Debug)]
pub struct VramView<'a> {
    renderer:     Arc<Mutex<TileRenderer>>,
    gfx_bufs:     GfxBuffers,
    viewed_tiles: ViewedVramTiles,
    selection:    Option<&'a mut (u32, u32)>,
}

impl<'a> VramView<'a> {
    pub fn new(renderer: Arc<Mutex<TileRenderer>>, gfx_bufs: GfxBuffers) -> Self {
        Self { renderer, gfx_bufs, viewed_tiles: ViewedVramTiles::All, selection: None }
    }

    pub fn viewed_tiles(mut self, viewed_tiles: ViewedVramTiles) -> Self {
        self.viewed_tiles = viewed_tiles;
        self
    }

    pub fn selection(mut self, selection: &'a mut (u32, u32)) -> Self {
        self.selection = Some(selection);
        self
    }

    pub fn new_renderer(gl: &Context) -> TileRenderer {
        let tiles = (0..16 * 64)
            .map(|t| {
                let scale = 16;
                let pos_x = (t % 16) * scale;
                let pos_y = (t / 16) * scale;
                let (tile, pal) = if t < 16 * 32 {
                    // background tiles
                    (t & 0x3FF, (t >> 10) & 0x7)
                } else {
                    // sprite tiles
                    ((t & 0x1FF) + 0x600, ((t >> 9) & 0x7) + 8)
                };
                let params = scale | (pal << 8) | (t & 0xC000);
                Tile([pos_x, pos_y, tile, params])
            })
            .collect_vec();
        let mut renderer = TileRenderer::new(gl);
        renderer.set_tiles(gl, tiles);
        renderer
    }
}

impl Widget for VramView<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self { renderer, gfx_bufs, viewed_tiles, selection } = self;
        let scale = tweak!(16.);
        let (height, offset) = match viewed_tiles {
            ViewedVramTiles::All => (64., Vec2::ZERO),
            ViewedVramTiles::BackgroundOnly => (32., Vec2::ZERO),
            ViewedVramTiles::SpritesOnly => (32., vec2(0., -32. * scale)),
        };
        let px = ui.ctx().pixels_per_point();
        let scale = scale / px;

        let rect_size = vec2(16., height) * scale;
        let (rect, response) =
            ui.allocate_exact_size(rect_size, if selection.is_some() { Sense::click() } else { Sense::hover() });

        let screen_size = rect.size() * px;
        ui.painter().add(PaintCallback {
            rect,
            callback: Arc::new(CallbackFn::new(move |_info, painter| {
                renderer.lock().expect("Cannot lock mutex on VRAM renderer").paint(
                    painter.gl(),
                    gfx_bufs,
                    screen_size,
                    offset,
                );
            })),
        });

        if let Some(selection) = selection {
            if let Some(hover_pos) = response.hover_pos() {
                let relative_pos = hover_pos - rect.left_top();
                let hovered_tile = (relative_pos / scale).floor().clamp(vec2(0., 0.), vec2(15., 31.));

                let hover_rect = Rect::from_min_size(
                    (hovered_tile * scale + rect.left_top().to_vec2()).to_pos2(),
                    Vec2::splat(scale),
                );
                ui.painter().rect_filled(
                    hover_rect,
                    Rounding::same(tweak!(3.)),
                    Color32::from_white_alpha(tweak!(100)),
                );

                if response.clicked() {
                    *selection = (hovered_tile.x as _, hovered_tile.y as _);
                }
            }

            let selection_rect = Rect::from_min_size(
                (vec2(selection.0 as f32 * scale, selection.1 as f32 * scale) + rect.left_top().to_vec2()).to_pos2(),
                Vec2::splat(scale),
            );
            ui.painter().rect_stroke(
                selection_rect,
                Rounding::same(tweak!(3.)),
                Stroke::new(tweak!(2.), Color32::from_rgba_premultiplied(200, 100, 30, 100)),
            );
        }

        response
    }
}
