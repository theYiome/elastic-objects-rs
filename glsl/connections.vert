#version 420
precision mediump float;

in vec2 local_position;
in vec2 position_a;
in vec2 position_b;
in float width;
in vec3 color;

uniform float screen_ratio = 1.0;
uniform float zoom = 1.0;
uniform vec2 camera_position = vec2(0.0, 0.0);

out vec3 outcolor;

void main() {
    outcolor = color;
    float len = length(position_b - position_a);
    vec2 position = position_a + (position_b - position_a) * 0.5;
    vec2 new_position = local_position * vec2(len, width) + position;
    new_position.y *= screen_ratio;
    gl_Position = vec4((new_position + camera_position) * zoom, 0.1, 1.0);
}