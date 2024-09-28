use egui::{util::id_type_map::SerializableAny, Color32, Context, Id};
use serde::{Deserialize, Serialize};

pub trait EditorStyle: Default + SerializableAny {
    fn id() -> Id;

    fn get_from_egui<R, F>(ctx: &Context, mut writer: F) -> R
    where
        F: FnMut(&mut Self) -> R,
    {
        ctx.data_mut(|data| writer(data.get_persisted_mut_or_default(Self::id())))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ErrorStyle {
    pub text_color: Color32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CellSelectorStyle {
    pub hovered_void_highlight_color: Color32,
    pub hovered_tile_highlight_color: Color32,
    pub selection_highlight_color:    Color32,
    pub selection_outline_color:      Color32,
    pub delete_highlight_color:       Color32,
}

impl EditorStyle for ErrorStyle {
    fn id() -> Id {
        Id::new("error_style")
    }
}

impl EditorStyle for CellSelectorStyle {
    fn id() -> Id {
        Id::new("cell_selector_style")
    }
}

impl Default for ErrorStyle {
    fn default() -> Self {
        Self { text_color: Color32::RED }
    }
}

impl Default for CellSelectorStyle {
    fn default() -> Self {
        Self {
            hovered_void_highlight_color: Color32::from_rgba_premultiplied(50, 50, 0, 30),
            hovered_tile_highlight_color: Color32::from_white_alpha(100),
            selection_highlight_color:    Color32::from_white_alpha(40),
            selection_outline_color:      Color32::from_rgba_premultiplied(255, 0, 0, 10),
            delete_highlight_color:       Color32::from_rgba_premultiplied(255, 0, 0, 10),
        }
    }
}
