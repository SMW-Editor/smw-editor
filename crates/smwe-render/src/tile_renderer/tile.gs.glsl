#version 400
layout(points) in;
in int v_tile_id[1];
in int v_params[1];

uniform vec2 screen_size;

layout(triangle_strip, max_vertices = 4) out;
flat out int g_tile_id;
flat out int g_params;
out vec2 g_tex_coords;

void main() {
	vec2 position = gl_in[0].gl_Position.xy;

	g_tile_id = v_tile_id[0];
	g_params = v_params[0];
    float scale = float(v_params[0] & 0xFF);

	vec2 pos;
	vec2 p;

	g_tex_coords = vec2(0.0, 0.0);
	pos = position + vec2(0.0, 0.0);
	p = (pos / screen_size * 2.0 - 1.0) * vec2(1.0, -1.0);
	gl_Position = vec4(p, 0.0, 1.0);
	EmitVertex();

	g_tex_coords = vec2(scale, 0.0);
	pos = position + vec2(scale, 0.0);
	p = (pos / screen_size * 2.0 - 1.0) * vec2(1.0, -1.0);
	gl_Position = vec4(p, 0.0, 1.0);
	EmitVertex();

	g_tex_coords = vec2(0.0, scale);
	pos = position + vec2(0.0, scale);
	p = (pos / screen_size * 2.0 - 1.0) * vec2(1.0, -1.0);
	gl_Position = vec4(p, 0.0, 1.0);
	EmitVertex();

	g_tex_coords = vec2(scale);
	pos = position + vec2(scale);
	p = (pos / screen_size * 2.0 - 1.0) * vec2(1.0, -1.0);
	gl_Position = vec4(p, 0.0, 1.0);
	EmitVertex();

	EndPrimitive();
}
