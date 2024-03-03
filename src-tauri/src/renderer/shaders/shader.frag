#version 450

layout(binding = 0) uniform UniformObject {
    vec4 color;
};

layout(location = 0) out vec4 outColor;

void main() {
    outColor = color;
}