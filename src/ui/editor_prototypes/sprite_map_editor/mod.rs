mod internals;
mod layout;

use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use egui::*;
use glow::Context;
use smwe_render::{
    gfx_buffers::GfxBuffers,
    tile_renderer::{Tile, TileRenderer},
};
use smwe_widgets::vram_view::VramView;

use crate::ui::editing_mode::EditingMode;

pub struct UiSpriteMapEditor {
    gl:              Arc<Context>,
    tile_palette:    Vec<Tile>,
    vram_renderer:   Arc<Mutex<TileRenderer>>,
    sprite_renderer: Arc<Mutex<TileRenderer>>,
    gfx_bufs:        GfxBuffers,

    level_num:    u16,
    editing_mode: EditingMode,

    initialized:      bool,
    scale:            f32,
    zoom:             f32,
    pixels_per_point: f32,

    selected_vram_tile:           (u32, u32),
    sprite_tiles:                 Vec<Tile>,
    last_inserted_tile:           Pos2,
    selected_sprite_tile_indices: HashSet<usize>,
}

impl UiSpriteMapEditor {
    pub fn new(gl: Arc<Context>) -> Self {
        let (vram_renderer, tile_palette) = VramView::new_renderer(&gl);
        let sprite_renderer = TileRenderer::new(&gl);
        let gfx_bufs = GfxBuffers::new(&gl);
        Self {
            gl,
            sprite_tiles: Vec::new(),
            vram_renderer: Arc::new(Mutex::new(vram_renderer)),
            sprite_renderer: Arc::new(Mutex::new(sprite_renderer)),
            gfx_bufs,
            level_num: 0,
            editing_mode: EditingMode::Move(None),
            initialized: false,
            scale: 8.,
            zoom: 2.,
            pixels_per_point: 0.,
            selected_vram_tile: (0, 0),
            tile_palette,
            last_inserted_tile: pos2(-1., -1.),
            selected_sprite_tile_indices: HashSet::new(),
        }
    }

    fn destroy(&self) {
        self.vram_renderer.lock().expect("Cannot lock mutex on VRAM renderer").destroy(&self.gl);
    }
}
