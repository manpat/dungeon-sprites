#version 450


// layout(std140, row_major, binding = 0) uniform UniformData {
// 	mat4 u_projection_view;
// };


// layout(location=0) in vec3 a_pos;
// layout(location=1) in vec4 a_color;
// layout(location=2) in vec2 a_uv;

// out vec4 v_color;
// out vec2 v_uv;


// void main() {
// 	gl_Position = u_projection_view * vec4(a_pos, 1.0);
// 	v_color = a_color;
// 	v_uv = a_uv;
// }


out vec2 v_uv;
out vec4 v_color;

const vec2[4] g_uvs = {
	{0.0, 0.0},
	{1.0, 0.0},
	{1.0, 1.0},
	{0.0, 1.0},
};

const vec2[4] g_positions = {
	{-1.0, -1.0},
	{1.0, -1.0},
	{1.0, 1.0},
	{-1.0, 1.0},
};

const uint g_indices[6] = {0, 1, 2, 0, 2, 3};

void main() {
	const uint index = g_indices[gl_VertexID % 6];

	gl_Position = vec4(g_positions[index], 0.0, 1.0);
	v_uv = g_uvs[index];
	v_color = vec4(1.0);
}