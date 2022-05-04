#version 450

layout (push_constant) uniform block { 
    mat4 projection;
    mat4 view;
    mat4 model;
};
layout (location = 0) in vec3 inPosition;
layout (location = 2) in vec3 inNormal;
layout (location = 3) in vec2 inUV;

layout (location = 0) out vec3 outColor;
layout (location = 1) out vec2 outUV;

void main() {
    gl_Position = projection * view * model * vec4(inPosition, 1.0);
    outColor = normalize(inNormal + vec3(0., 0., 2.));
    outUV = inUV;
}