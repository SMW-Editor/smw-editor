use std::path::PathBuf;

use itertools::Itertools;
use rfd::{MessageButtons, MessageDialog, MessageLevel};
use smwe_render::tile_renderer::{Tile, TileJson};

use super::super::{SpriteTiles, UiSpriteMapEditor};

impl UiSpriteMapEditor {
    pub(in super::super) fn create_new_map(&mut self) {
        self.sprite_tiles.write(|tiles| tiles.clear());
        self.upload_tiles();
    }

    pub(in super::super) fn open_map(&mut self, path: PathBuf) {
        match std::fs::read_to_string(path) {
            Err(e) => {
                MessageDialog::new()
                    .set_title("Failed to open selected file.")
                    .set_description(&format!("{e:?}"))
                    .set_level(MessageLevel::Error)
                    .set_buttons(MessageButtons::Ok)
                    .show();
            }
            Ok(s) => match serde_json::from_str::<Vec<TileJson>>(&s) {
                Err(e) => {
                    MessageDialog::new()
                        .set_title("Failed to deserialize sprite tile map from JSON.")
                        .set_description(&format!("{e:?}"))
                        .set_level(MessageLevel::Error)
                        .set_buttons(MessageButtons::Ok)
                        .show();
                }
                Ok(loaded_tiles) => {
                    self.sprite_tiles.write(move |tiles| {
                        *tiles = SpriteTiles(loaded_tiles.into_iter().map(Tile::from).collect_vec());
                    });
                    self.sprite_tiles.clear_stack();
                    self.selected_sprite_tile_indices.clear();
                    self.upload_tiles();
                }
            },
        }
    }

    pub(in super::super) fn save_map_as(&mut self, path: PathBuf) {
        let tiles = self.sprite_tiles.read(|tiles| tiles.iter().map(|&t| TileJson::from(t)).collect_vec());
        match serde_json::to_string_pretty(&tiles) {
            Err(e) => {
                MessageDialog::new()
                    .set_title("Failed to serialize sprite tile map into JSON.")
                    .set_description(&format!("{e:?}"))
                    .set_level(MessageLevel::Error)
                    .set_buttons(MessageButtons::Ok)
                    .show();
            }
            Ok(s) => {
                if let Err(e) = std::fs::write(path, s) {
                    MessageDialog::new()
                        .set_title("Save sprite tile map to selected file.")
                        .set_description(&format!("{e:?}"))
                        .set_level(MessageLevel::Error)
                        .set_buttons(MessageButtons::Ok)
                        .show();
                }
            }
        }
    }
}
