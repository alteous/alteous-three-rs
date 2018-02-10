#version 150 core

layout(std140) uniform b_Locals {
    mat4 u_World;
    vec4 u_Color;
};

in vec2 v_TexCoord;

uniform sampler2D t_Map;

void main() {
    gl_FragColor = u_Color; //* texture(t_Map, v_TexCoord);
}
