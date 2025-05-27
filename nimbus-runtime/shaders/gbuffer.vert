#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(location = 0) out vec3 fragPos;
layout(location = 1) out vec3 fragNormal;
layout(location = 2) out vec2 fragTexCoord;

layout(set = 0, binding = 0) uniform Matrices {
    mat4 model;
    mat4 view;
    mat4 projection;
} matrices;

void main() {
    mat4 modelView = matrices.view * matrices.model;
    fragPos = vec3(modelView * vec4(position, 1.0));
    fragNormal = mat3(transpose(inverse(modelView))) * normal;
    fragTexCoord = uv;
    gl_Position = matrices.projection * vec4(fragPos, 1.0);
}