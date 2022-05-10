#version 420
precision mediump float;

in vec2 local_position;

uniform float screen_ratio = 1.0;
uniform float zoom = 1.0;
uniform vec2 camera_position = vec2(0.0, 0.0);

out vec3 outcolor;

void main() {
    outcolor = vec3(0.1, 0.1, 0.1);
    vec2 new_position = local_position;
    new_position.y *= screen_ratio;
    gl_Position = vec4((new_position + camera_position) * zoom, 0.2, 1.0);
}