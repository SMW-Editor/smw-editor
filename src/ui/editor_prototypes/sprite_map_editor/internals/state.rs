use egui::Context;

use super::super::UiSpriteMapEditor;

impl UiSpriteMapEditor {
    pub(in super::super) fn reset_state(&mut self, ctx: &Context) {
        if self.state_needs_reset {
            self.update_cpu();
            self.update_renderers();
            self.pixels_per_point = ctx.pixels_per_point();
            self.state_needs_reset = false;
        }
    }

    pub(in super::super) fn update_cpu(&mut self) {
        smwe_emu::emu::decompress_sublevel(&mut self.cpu, self.level_num);
        println!("Updated CPU");
    }

    pub(in super::super) fn update_renderers(&mut self) {
        self.gfx_bufs.upload_palette(&self.gl, &self.cpu.mem.cgram);
        self.gfx_bufs.upload_vram(&self.gl, &self.cpu.mem.vram);
    }

    pub(in super::super) fn upload_tiles(&self) {
        self.sprite_renderer
            .lock()
            .expect("Cannot lock mutex on sprite renderer")
            .set_tiles(&self.gl, self.sprite_tiles.read(|tiles| tiles.0.clone()));
    }

    pub(in super::super) fn update_tile_palette(&mut self) {
        for tile in self.tile_palette.iter_mut() {
            tile[3] &= 0xC0FF;
            tile[3] |= (self.selected_palette + 8) << 8;
        }
        self.vram_renderer
            .lock()
            .expect("Cannot lock mutex on VRAM renderer")
            .set_tiles(&self.gl, self.tile_palette.clone());
    }
}
