#version 330
#define MAX_POINT_LIGHTS 8

struct AmbientLight {
    vec3 color;
    float intensity;
};

struct DirectionalLight {
    vec4 position;
    vec3 direction;
    vec3 color;
    float intensity;
};

struct PointLight {
    vec4 position;
    vec3 color;
    float intensity;
};

layout(std140) uniform b_Globals {
    mat4 u_ViewProj;
    AmbientLight u_AmbientLight;
    DirectionalLight u_DirectionalLight;
};

layout(std140) uniform b_Locals {
    mat4 u_World;
    float u_Glossiness;
    PointLight u_PointLights[MAX_POINT_LIGHTS];
};

layout(location = 0) in vec4 a_Position;
layout(location = 2) in vec4 a_Normal;

out vec3 v_Position;
out vec3 v_Normal;

void main() {
    vec4 world_position = u_World * a_Position;
    v_Position = world_position.xyz;
    v_Normal = normalize(mat3(u_World) * a_Normal.xyz);
    gl_Position = u_ViewProj * world_position;
}
