#version 400

layout(location = 0) in vec4 a_Position;
layout(location = 1) in vec4 a_TexCoord;
layout(location = 2) in vec4 a_Normal;
layout(location = 3) in vec4 a_Tangent;
layout(location = 4) in uvec4 a_JointIndices;
layout(location = 5) in vec4 a_JointWeights;

layout(std140) uniform b_Globals {
    mat4 u_World;
    mat4 u_ViewProjection;
};

out float v_Depth;

void main()
{
    vec4 clip_position = u_ViewProjection * u_World * a_Position;
    v_Depth = clip_position.z / clip_position.w;
    gl_Position = u_ViewProjection * u_World * a_Position;
}
