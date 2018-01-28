#version 150 core

in vec2 a_pos;
in vec2 a_uv;
out vec2 v_uv;

void main() {
	v_uv = a_uv;
	gl_Position = vec4(a_pos, 0.0, 1.0);
}