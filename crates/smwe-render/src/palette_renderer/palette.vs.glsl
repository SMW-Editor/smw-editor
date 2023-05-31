#version 400

layout (location = 0) in int vertex_index;

out vec2 v_tex_coords;

void main() {
    int x = vertex_index & 2;
    int y = vertex_index & 1;
    v_tex_coords = vec2(x, y);
    gl_Position = vec4((2 * x) - 1, 1 - (2 * y), 0, 1);
}
