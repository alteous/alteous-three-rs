#version 140

layout(std140) uniform b_Locals {
    mat4 u_World;
    vec4 u_Color;
};

in float v_Depth;

void main()
{
    gl_FragColor = vec4(v_Depth, 0.0, 0.0, 0.0);
}
