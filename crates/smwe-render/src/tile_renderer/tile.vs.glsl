#version 400
layout (location = 0) in ivec4 position;

uniform vec2 screen_size;
uniform vec2 offset;
uniform float zoom;

out int v_tile_id;
out int v_params;

void main() {
    v_tile_id = position.z;
    v_params = position.w;
    gl_Position = vec4((vec2(position.xy) + offset) * zoom, 0.0, 1.0);
}
