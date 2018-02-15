#version 150 core
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

in vec4 v_ResultColor;
flat in vec4 v_ResultColorFlat;
flat in float v_Smooth;

void main()
{
    gl_FragColor = mix(v_ResultColorFlat, v_ResultColor, v_Smooth);
}
