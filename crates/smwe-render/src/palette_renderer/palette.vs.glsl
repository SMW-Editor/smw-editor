#version 400

#define VIEW_ALL_PALETTES        0
#define VIEW_BACKGROUND_PALETTES 1
#define VIEW_SPRITE_PALETTES     2

layout (location = 0) in int vertex_index;

uniform uint u_viewed_palettes;

out vec2 v_tex_coords;

void main() {
    float x = float(vertex_index & 2);
    float y = float(vertex_index & 1);
    
    switch (u_viewed_palettes) {
        case VIEW_ALL_PALETTES:
            v_tex_coords = vec2(x, y);
            break;
        case VIEW_BACKGROUND_PALETTES:
            v_tex_coords = vec2(x, y * 0.5);
            break;
        case VIEW_SPRITE_PALETTES:
            v_tex_coords = vec2(x, (y * 0.5) + 0.5);
            break;
        default:
            v_tex_coords = vec2(0);
            break;
    }
    
    gl_Position = vec4((2 * x) - 1, 1 - (2 * y), 0, 1);
}
