#version 450

layout(location = 0) in vec3 fragPos;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in vec2 fragTexCoord;

layout(location = 0) out vec4 outPosition;
layout(location = 1) out vec4 outNormal;
layout(location = 2) out vec4 outAlbedo;
layout(location = 3) out vec4 outMaterial;

layout(set = 1, binding = 0) uniform sampler2D albedoMap;
layout(set = 1, binding = 1) uniform sampler2D metallicRoughnessMap;

void main() {
    vec3 albedo = texture(albedoMap, fragTexCoord).rgb;
    vec2 mr = texture(metallicRoughnessMap, fragTexCoord).rg;

    outPosition = vec4(fragPos, 1.0);
    outNormal = vec4(normalize(fragNormal), 0.0);
    outAlbedo = vec4(albedo, 1.0);
    outMaterial = vec4(mr.r, mr.g, 0.0, 0.0);
}
