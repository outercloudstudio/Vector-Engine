#version 450

layout(binding = 0) uniform UniformObject {
    vec4 color;
    vec2 position;
    vec2 origin;
    vec2 size;
    float radius;
    float rotation;
};

layout(location = 1) in vec2 uv;

layout(location = 0) out vec4 outColor;

void main() {
    if(distance(uv, vec2(0.5, 0.5)) > 0.5) discard;

    outColor = color;
}