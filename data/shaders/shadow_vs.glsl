#version 150

layout(std140) uniform b_Locals {
    mat4 u_ModelViewProjection;
};

in vec4 a_Position;

void main() {
    gl_Position = u_ModelViewProjection * a_Position;
}
