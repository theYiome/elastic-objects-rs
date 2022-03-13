#version 300 es
precision mediump float;

in vec2 local_position;
in vec2 position;
in float scale_x;
in float scale_y;
in float rotation;
in vec3 color;

out vec3 outcolor;

void main() {
    outcolor = color;
    gl_Position = vec4(local_position * vec2(scale_x, scale_y) + position, 0.0, 1.0);
}