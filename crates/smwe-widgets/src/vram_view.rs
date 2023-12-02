use std::sync::{Arc, Mutex};

use egui::{vec2, Color32, PaintCallback, PointerButton, Rect, Response, Rounding, Sense, Stroke, Ui, Vec2, Widget};
use egui_glow::{glow::Context, CallbackFn};
use inline_tweak::tweak;
use itertools::Itertools;
use smwe_render::{
    gfx_buffers::GfxBuffers,
    tile_renderer::{Tile, TileRenderer, TileUniforms},
};

#[derive(Copy, Clone, Debug)]
pub enum ViewedVramTiles {
    All,
    BackgroundOnly,
    SpritesOnly,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VramSelectionMode {
    SingleTile,
    TwoByTwoTiles,
}

#[derive(Debug)]
pub struct VramView<'a> {
    renderer:     Arc<Mutex<TileRenderer>>,
    gfx_bufs:     GfxBuffers,
    viewed_tiles: ViewedVramTiles,
    selection:    Option<(VramSelectionMode, &'a mut (u32, u32))>,
    zoom:         f32,
}

impl<'a> VramView<'a> {
    pub fn new(renderer: Arc<Mutex<TileRenderer>>, gfx_bufs: GfxBuffers) -> Self {
        Self { renderer, gfx_bufs, viewed_tiles: ViewedVramTiles::All, selection: None, zoom: 1. }
    }

    pub fn viewed_tiles(mut self, viewed_tiles: ViewedVramTiles) -> Self {
        self.viewed_tiles = viewed_tiles;
        self
    }

    pub fn selection(mut self, mode: VramSelectionMode, selection: &'a mut (u32, u32)) -> Self {
        self.selection = Some((mode, selection));
        self
    }

    pub fn zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom;
        self
    }

    pub fn new_renderer(gl: &Context) -> (TileRenderer, Vec<Tile>) {
        let tiles = (0..16 * 64)
            .map(|t| {
                let scale = 8;
                let pos_x = (t % 16) * scale;
                let pos_y = (t / 16) * scale;
                let (tile, pal) = if t < 16 * 32 {
                    // background tiles
                    (t & 0x3FF, 0)
                } else {
                    // sprite tiles
                    ((t & 0x1FF) + 0x600, 8)
                };
                let params = scale | (pal << 8) | (t & 0xC000);
                Tile([pos_x, pos_y, tile, params])
            })
            .collect_vec();
        let mut renderer = TileRenderer::new(gl);
        renderer.set_tiles(gl, tiles.clone());
        (renderer, tiles)
    }
}

impl Widget for VramView<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self { renderer, gfx_bufs, viewed_tiles, selection, zoom } = self;
        let scale = tweak!(8.);
        let (height, offset) = match viewed_tiles {
            ViewedVramTiles::All => (64., Vec2::ZERO),
            ViewedVramTiles::BackgroundOnly => (32., Vec2::ZERO),
            ViewedVramTiles::SpritesOnly => (32., vec2(0., -32. * scale)),
        };
        let px = ui.ctx().pixels_per_point();
        let scale = scale / px;

        let rect_size = vec2(16., height) * scale * zoom;
        let (rect, response) =
            ui.allocate_exact_size(rect_size, if selection.is_some() { Sense::click() } else { Sense::hover() });

        // VRAM image
        let screen_size = rect.size() * px;
        ui.painter().add(PaintCallback {
            rect,
            callback: Arc::new(CallbackFn::new(move |_info, painter| {
                renderer.lock().expect("Cannot lock mutex on VRAM renderer").paint(painter.gl(), &TileUniforms {
                    gfx_bufs,
                    screen_size,
                    offset,
                    zoom,
                });
            })),
        });

        // Hover/select tile
        if let Some((mode, selection)) = selection {
            let (selection_size, max_selected_tile) = match mode {
                VramSelectionMode::SingleTile => (scale, vec2(15., 31.)),
                VramSelectionMode::TwoByTwoTiles => (2. * scale, vec2(14., 30.)),
            };

            let selection_rect = Rect::from_min_size(rect.left_top(), Vec2::splat(selection_size * zoom));

            if let Some(hover_pos) = response.hover_pos() {
                let relative_pos = hover_pos - rect.left_top();
                let hovered_tile = (relative_pos / scale / zoom).floor().clamp(vec2(0., 0.), max_selected_tile);

                ui.painter().rect_filled(
                    selection_rect.translate(hovered_tile * scale * zoom),
                    Rounding::same(tweak!(3.)),
                    Color32::from_white_alpha(tweak!(100)),
                );

                if response.clicked_by(PointerButton::Primary) {
                    *selection = (hovered_tile.x as _, hovered_tile.y as _);
                }
            }

            ui.painter().rect_stroke(
                selection_rect.translate(vec2(selection.0 as f32, selection.1 as f32) * scale * zoom),
                Rounding::same(tweak!(3.)),
                Stroke::new(tweak!(2.), Color32::from_rgba_premultiplied(200, 100, 30, 100)),
            );
        }

        response
    }
}
