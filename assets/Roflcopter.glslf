#version 150 core

in vec2 v_uv;
out vec4 Target0;
uniform sampler2D t_color;

void main()
{
    vec2 resolution = vec2(1440, 855);
    vec2 direction = vec2(0.5, -1.0);
    vec3 color0 = vec3(0.667, 0.643, 0.24);
    vec3 color1 = vec3(0.471, 0.62, 0.208);
    vec3 color2 = vec3(0.545, 0.18, 0.373);
    vec3 color = vec3(0.0);
    vec2 off1 = vec2(1.5333333333333333);// * direction;
    color += color0 * texture(t_color, v_uv).rgb * 0.29411764705882354;
    color += color1 * texture(t_color, v_uv + (off1 / resolution)).rgb * 0.35294117647058826;
    color += color2 * texture(t_color, v_uv - (off1 / resolution)).rgb * 0.35294117647058826;
    color.rgb /= color0 + color1 + color2;

	Target0 = vec4(color, 1.0);
}