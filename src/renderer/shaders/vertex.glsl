#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 texture;

layout(set = 0, binding = 0) uniform SceneData {
    vec2 size;
} scene;

layout(push_constant) uniform MeshData {
    vec2 offset;
} mesh;

layout(constant_id = 0) const float quad_scale = 100;

layout(location = 0) out vec2 uv;

void main() {
    vec2 scaled = position * quad_scale;
    vec2 offset = scaled + mesh.offset;
    vec2 adjusted = 2.0 * offset / scene.size - 1.0;

    gl_Position = vec4(adjusted, 0.0, 1.0);
    uv = texture;
}