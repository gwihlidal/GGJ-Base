#version 150 core

in vec2 v_uv;
out vec4 Target0;
uniform sampler2D t_color;

void main() {
	vec4 tex = texture(t_color, v_uv);
	Target0 = tex;
}