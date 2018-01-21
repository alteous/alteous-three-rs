#version 400

layout(location = 0) in vec4 a_Position;
layout(location = 1) in vec4 a_TexCoord;
layout(location = 2) in vec4 a_Normal;
layout(location = 3) in vec4 a_Tangent;
layout(location = 4) in uvec4 a_JointIndices;
layout(location = 5) in vec4 a_JointWeights;

void main()
{
    gl_Position = a_Position;
}