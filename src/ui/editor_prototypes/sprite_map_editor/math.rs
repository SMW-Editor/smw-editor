use egui::emath::*;
use smwe_render::tile_renderer::Tile;

pub(super) fn tile_contains_point(tile: &Tile, point: Pos2, scale: f32) -> bool {
    Rect::from_min_size((tile.pos().to_vec2() * scale).to_pos2(), Vec2::splat(tile.scale() as f32 * scale))
        .contains(point)
}

pub(super) fn tile_intersects_rect(tile: &Tile, rect: Rect, scale: f32) -> bool {
    Rect::from_min_size((tile.pos().to_vec2() * scale).to_pos2(), Vec2::splat(tile.scale() as f32 * scale))
        .intersects(rect)
}

pub(super) fn point_to_tile_coords(pos_in_canvas: Pos2, scale_pp: f32, zoom: f32) -> Vec2 {
    (pos_in_canvas.to_vec2() / scale_pp / zoom).floor().clamp(vec2(0., 0.), vec2(31., 31.))
}
