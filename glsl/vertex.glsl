#version 140

in vec2 position;
in vec3 col;

out vec3 outcolor;

void main() {
    outcolor = col;
    gl_Position = vec4(position, 0.0, 1.0);
}