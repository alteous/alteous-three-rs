#version 150 core

layout(std140) uniform b_Locals {
    mat4 u_World;
    vec4 u_Color;
};

in vec2 v_TexCoord;

uniform sampler2D t_Map;

void main() {
    gl_FragColor = vec4(1.0, 1.0, 0.0, 1.0); // u_Color; //* texture(t_Map, v_TexCoord);
}
