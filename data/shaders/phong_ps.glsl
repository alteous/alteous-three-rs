#version 150 
#define MAX_POINT_LIGHTS 8

struct AmbientLight {
    vec3 color;
    float intensity;
};

struct DirectionalLight {
    vec4 position;
    vec3 direction;
    vec3 color;
    float intensity;
};

struct PointLight {
    vec4 position;
    vec3 color;
    float intensity;
};

layout(std140) uniform b_Globals {
    mat4 u_ViewProj;
    AmbientLight u_AmbientLight;
    DirectionalLight u_DirectionalLight;
};

layout(std140) uniform b_Locals {
    mat4 u_World;
    float u_Glossiness;
    PointLight u_PointLights[MAX_POINT_LIGHTS];
};

in vec3 v_Position;
in vec3 v_Normal;

void main() {
    vec3 frag_normal = normalize(v_Normal);
    vec3 ambient = u_AmbientLight.color * u_AmbientLight.intensity;
    vec3 diffuse = vec3(0.0);
    vec3 specular = vec3(0.0);
    for (int i = 0; i < MAX_POINT_LIGHTS; ++i) {
	vec3 light_color = u_PointLights[i].intensity * u_PointLights[i].color;
	vec3 light_dir = normalize(u_PointLights[i].position.xyz - v_Position);
	diffuse += light_color * max(0.0, dot(frag_normal, light_dir));
    }
    gl_FragColor = vec4(max(ambient, diffuse + specular), 1.0);
}
