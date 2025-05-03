#version 450

layout(binding = 0) uniform UniformObject {
    vec4 color;
    vec2 position;
    vec2 origin;
    vec2 size;
    float radius;
    float rotation;
};

layout(location = 0) in vec2 vertex_position;
layout(location = 1) in vec2 uv;

layout(location = 1) out vec2 outUv;

void main() {
    vec2 pixel_position = vertex_position * size - origin * size;
    vec2 offset = pixel_position;
    vec2 rotated_position = position + vec2(offset.x * cos(rotation) - offset.y * sin(rotation), offset.y * cos(rotation) + offset.x * sin(rotation));
    vec2 scaled_position = rotated_position / vec2(1920.0 / 2.0, 1080.0 / 2.0);
    vec2 flipped_position = vec2(scaled_position.x, -scaled_position.y);

    gl_Position = vec4(flipped_position, 0, 1.0);
    outUv = uv;
}