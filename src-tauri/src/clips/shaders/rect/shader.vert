#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(location = 1) out vec2 outUv;

void main() {
    gl_Position = vec4(position, 0, 1.0);
    outUv = uv;
}
