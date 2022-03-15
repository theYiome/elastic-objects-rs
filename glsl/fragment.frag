#version 460
precision mediump float;

in vec3 outcolor;
out vec4 color;

uniform float tim;

void main() {
    color = vec4(outcolor, 1.0);
}