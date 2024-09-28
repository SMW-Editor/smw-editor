#version 400

in vec2 v_tex_coords;

layout(std140) uniform Color {
    vec4 colors[0x100];
};

out vec4 out_color;

void main() {
    uint color_col = uint(v_tex_coords.x * 0x10);
    uint color_row = uint(v_tex_coords.y * 0x10);
    if (color_col == 0) {
        discard;
    } else {
        uint color_idx = color_col + color_row * 0x10;
        out_color = colors[color_idx];
    }
}
