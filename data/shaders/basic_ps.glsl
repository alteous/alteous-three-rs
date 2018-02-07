#version 150 core

layout(std140) uniform b_Locals {
    vec4 u_Color;
    mat4 u_World;
};

in vec2 v_TexCoord;

uniform sampler2D t_Map;

void main() {
    gl_FragColor = vec4(1.0, 1.0, 0.0, 0.0);//u_Color * texture(t_Map, v_TexCoord);
}
