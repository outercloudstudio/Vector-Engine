#version 450

layout(binding = 0) uniform UniformObject {
    vec4 color;
    vec2 size;
    float radius;
};

layout(location = 1) in vec2 uv;

layout(location = 0) out vec4 outColor;

void main() { 
    vec2 pixelUv = uv * size;

    if(pixelUv.x < radius && pixelUv.y < radius && distance(pixelUv, vec2(radius, radius)) > radius) discard;
    if(pixelUv.x > size.x - radius && pixelUv.y < radius && distance(pixelUv, vec2(size.x - radius, radius)) > radius) discard;
    if(pixelUv.x < radius && pixelUv.y > size.y - radius && distance(pixelUv, vec2(radius, size.y - radius)) > radius) discard;
    if(pixelUv.x > size.x - radius && pixelUv.y > size.y - radius && distance(pixelUv, vec2(size.x- radius, size.y - radius)) > radius) discard;

    outColor = color;
}