#version 330 core

layout (location = 0) in vec3 inPosition;
layout (location = 1) in vec3 inColor;
layout (location = 2) in vec2 inTexCoord;

out vec3 outColor;
out vec2 outTexCoord;

uniform mat4 transform;

void main()
{
    gl_Position = transform * vec4(inPosition, 1.0);
    outColor = inColor;
    outTexCoord = inTexCoord;
}
