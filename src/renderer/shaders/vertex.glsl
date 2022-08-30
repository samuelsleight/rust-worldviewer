#version 450

layout(location = 0) in vec2 position;

layout(set = 0, binding = 0) uniform SceneData {
    vec2 size;
} scene;

layout(push_constant) uniform MeshData {
    vec2 offset;
} mesh;

void main() {
    vec2 offset = position + mesh.offset;
    vec2 adjusted = 2.0 * offset / scene.size - 1.0;
    gl_Position = vec4(adjusted, 0.0, 1.0);
}