#version 150
#define MAX_POINT_LIGHTS 8

struct AmbientLight {
    vec3 color;
    float intensity;
};

struct DirectionalLight {
    vec3 direction;
    vec3 color;
    float intensity;
};

struct PointLight {
    vec3 position;
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
    vec3 u_Color;
    float u_Smooth;
    PointLight u_PointLights[MAX_POINT_LIGHTS];
};

layout(location = 0) in vec4 a_Position;
layout(location = 2) in vec4 a_Normal;

out vec4 v_ResultColor;
flat out vec4 v_ResultColorFlat;
flat out float v_Smooth;

void main()
{
    vec4 world = u_World * a_Position;
    vec3 vert_normal = normalize(mat3(u_World) * a_Normal.xyz);
    v_ResultColor = vec4(0.0);
    v_Smooth = u_Smooth;

    vec3 ambient = u_AmbientLight.color * u_AmbientLight.intensity;
    vec3 diffuse = vec3(0.0);
    for (int i = 0; i < MAX_POINT_LIGHTS; ++i) {
        vec3 light_color = u_PointLights[i].intensity * u_PointLights[i].color;
        vec3 light_dir = normalize(u_PointLights[i].position - a_Position.xyz);
        diffuse += light_color * max(0.0, dot(vert_normal, light_dir));
    }

    v_ResultColor = vec4(u_Color * max(ambient, diffuse), 1.0);
    v_ResultColorFlat = v_ResultColor;
    gl_Position = u_ViewProj * world;
}
