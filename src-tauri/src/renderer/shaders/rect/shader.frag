#version 450

layout(binding = 0) uniform UniformObject {
    vec4 color;
    float radius;
    vec2 size;
};

layout(location = 0) out vec4 outColor;

void main() {
    outColor = color;
}

/*
vec2 pixelUV = UV * size;

if(pixelUV.x < radius && pixelUV.y < radius && distance(pixelUV, vec2(radius, radius)) > radius) discard;
if(pixelUV.x > size.x - radius && pixelUV.y < radius && distance(pixelUV, vec2(size.x - radius, radius)) > radius) discard;
if(pixelUV.x < radius && pixelUV.y > size.y - radius && distance(pixelUV, vec2(radius, size.y - radius)) > radius) discard;
if(pixelUV.x > size.x - radius && pixelUV.y > size.y - radius && distance(pixelUV, vec2(size.x- radius, size.y - radius)) > radius) discard;
*/