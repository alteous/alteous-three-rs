#version 150 core

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

void main() {
    gl_Position = u_ViewProj * u_World * a_Position;
}
