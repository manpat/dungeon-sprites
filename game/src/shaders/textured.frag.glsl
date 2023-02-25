#version 450

in vec4 v_color;
in vec2 v_uv;

layout(binding=0) uniform sampler2D u_texture;

layout(location=0) out vec4 out_color;


float dither2x2(float value) {
	ivec2 position = ivec2(gl_FragCoord.xy);
	int x = position.x % 2;
	int y = position.y % 2;
	int index = x + y * 2;

	float limit = 0.0;
	if (x < 8) {
		if (index == 0) limit = 0.25;
		if (index == 1) limit = 0.75;
		if (index == 2) limit = 0.999;
		if (index == 3) limit = 0.50;
	}

	return step(limit, value);
}


void main() {
	vec4 tex_color = texture(u_texture, v_uv);
	out_color = tex_color * v_color;

	if (dither2x2(out_color.a) < 0.5) {
		discard;
	}
}