#version 330 core

layout(std140) uniform b_Locals {
    mat4 u_World;
    vec4 u_Color;
    vec4 u_UvRange;
};

layout(std140) uniform b_Globals {
    mat4 u_ViewProj;
    mat4 u_InverseProj;
    mat4 u_View;
    uint u_NumLights;
};

layout(location = 0) in vec4 a_Position;
layout(location = 1) in vec2 a_TexCoord;
layout(location = 2) in vec3 a_Normal;

out vec2 v_TexCoord;

void main() {
    v_TexCoord = mix(u_UvRange.xy, u_UvRange.zw, a_TexCoord);
    gl_Position = u_ViewProj * u_World * a_Position;
}
