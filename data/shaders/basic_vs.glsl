#version 150 core
#include <globals>

layout(std140) uniform b_Locals {
    vec4 u_Color;
    mat4 u_World;
};

in vec4 a_Position;
in vec4 a_Normal;
in vec2 a_TexCoord;
out vec2 v_TexCoord;

void main() {
    v_TexCoord = mix(u_UvRange.xy, u_UvRange.zw, a_TexCoord);
    gl_Position = u_ViewProj * u_World * a_Position;
}
