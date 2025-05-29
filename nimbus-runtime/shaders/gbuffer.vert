#version 450

layout(location = 0) in vec3 vPosition;
layout(location = 1) in vec3 vNormal;
layout(location = 2) in vec2 vTexCoord;

layout(location = 0) out vec3 fragPos;
layout(location = 1) out vec3 fragNormal;
layout(location = 2) out vec2 fragTexCoord;

layout(set = 0, binding = 0) uniform Matrices {
    mat4 view;
    mat4 projection;
    vec4 cameraPosition;
} matrices;

layout(push_constant) uniform PushConstants {
    mat4 model;
} pushConstants;

void main() {
    mat4 modelView = matrices.view * pushConstants.model;
    fragPos = vec3(modelView * vec4(vPosition, 1.0));
    fragNormal = mat3(transpose(inverse(modelView))) * vNormal;
    fragTexCoord = vTexCoord;
    gl_Position = matrices.projection * vec4(fragPos, 1.0);
}