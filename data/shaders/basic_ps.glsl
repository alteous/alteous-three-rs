#version 150 core
#include <locals>

in vec2 v_TexCoord;
out vec4 Target0;

uniform sampler2D t_Map;

void main() {
    Target0 = vec4(1.0, 1.0, 0.0, 0.0);//u_Color * texture(t_Map, v_TexCoord);
}
