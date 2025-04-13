#version 450

layout(location = 0) in vec2 uv;

layout(location = 0) out vec2 outUv;

vec2 positions[4] = vec2[](
    vec2(-1, -1),
    vec2(1, -1),
    vec2(1, 1),
    vec2(-1, 1)
);

void main() {
    gl_Position = vec4(positions[gl_VertexIndex], 0, 1.0);
    outUv = uv;
}
