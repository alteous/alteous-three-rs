#version 140

layout(std140) uniform UniformBlock {
    vec4 u_Color;
};

uniform sampler2D u_Sampler;

void main()
{
    gl_FragColor = vec4(1.0, 1.0, 0.0, 0.0);
}
