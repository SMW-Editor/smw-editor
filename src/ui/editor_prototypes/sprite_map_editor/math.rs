use egui::emath::*;
use smwe_render::tile_renderer::Tile;

pub(super) fn tile_contains_point(tile: Tile, point: Pos2, scale: f32) -> bool {
    let min = (tile.pos().to_vec2() * scale).to_pos2();
    let size = Vec2::splat(tile.scale() as f32 * scale);
    Rect::from_min_size(min, size).contains(point)
}

pub(super) fn tile_intersects_rect(tile: Tile, rect: Rect, scale: f32) -> bool {
    let min = (tile.pos().to_vec2() * scale).to_pos2();
    let size = Vec2::splat(tile.scale() as f32 * scale);
    Rect::from_min_size(min, size).intersects(rect)
}

pub(super) fn point_to_tile_coords(pos_in_canvas: Pos2, scale_pp: f32, zoom: f32) -> Vec2 {
    let unscaled_pos = pos_in_canvas.to_vec2() / scale_pp / zoom;
    unscaled_pos.floor().clamp(vec2(0., 0.), vec2(31., 31.))
}
