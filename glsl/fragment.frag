#version 300 es
precision mediump float;

in vec3 outcolor;
out vec4 color;

uniform float tim;

void main() {
    color = vec4(tim, outcolor.y, tim, 1.0);
}