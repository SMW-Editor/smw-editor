#version 400
flat in int g_tile_id;
flat in int g_params;
in vec2 g_tex_coords;

uniform vec2 screen_size;

layout(std140) uniform Graphics {
    uvec4 graphics[0x1000];
};
layout(std140) uniform Color {
    vec4 colors[0x100];
};

out vec4 out_color;

void main() {
    const int scale = 8;
    
    int tile_id = g_tile_id & 0x3FF;
    int color_row = (g_tile_id >> 10) & 0x7;
    ivec2 icoord = ivec2(g_tex_coords) * 8 / scale;
    
    bool flip_x = (g_tile_id & 0x4000) != 0;
    bool flip_y = (g_tile_id & 0x8000) != 0;
    
    if (flip_y) {
        icoord.y = 7 - icoord.y;
    }
    if (flip_x) {
        icoord.x = 7 - icoord.x;
    }

    uvec4 part1 = graphics[tile_id * 2 + 0];
    uvec4 part2 = graphics[tile_id * 2 + 1];

    uint lpart1 = part1[icoord.y / 2];
    uint lpart2 = part2[icoord.y / 2];

    int line1 = int(lpart1 >> ((icoord.y % 4) * 16));
    int line2 = int(lpart2 >> ((icoord.y % 4) * 16));

    int color_col = 0;
    color_col |= ((line1 >> ( 7 - icoord.x)) & 0x1) << 0;
    color_col |= ((line1 >> (15 - icoord.x)) & 0x1) << 1;
    color_col |= ((line2 >> ( 7 - icoord.x)) & 0x1) << 2;
    color_col |= ((line2 >> (15 - icoord.x)) & 0x1) << 3;
    
    if (color_col == 0) {
        discard;
//        out_color = vec4(1, 0, 0, 0);
    } else {
        uint color_idx = color_col + color_row * 0x10;
        out_color = colors[color_idx];
//        out_color = vec4(0, 1, 0, 0);
    }
}
