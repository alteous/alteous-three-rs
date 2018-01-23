#version 140

layout(std140) uniform b_Locals {
    vec4 u_Color;
};

void main()
{
    gl_FragColor = u_Color;
}
