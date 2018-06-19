#version 330 core

in vec3 outColor;
in vec2 outTexCoord;

out vec4 Color;

uniform sampler2D tex;

void main()
{
    Color = texture(tex, outTexCoord);
}
