#version 450

layout(binding = 0) uniform UniformObject {
    vec4 color;
    vec2 size;
};

layout(binding = 1) uniform sampler2D textureSampler;

layout(location = 1) in vec2 uv;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = texture(textureSampler, uv) * color;
}