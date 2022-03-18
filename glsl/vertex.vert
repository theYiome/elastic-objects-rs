#version 460
precision mediump float;

in vec2 local_position;
in vec2 position;
in float scale_x;
in float scale_y;
in float rotation;
in vec3 color;

uniform float screen_ratio = 1.0;
uniform float zoom = 1.0;
uniform vec2 camera_position = vec2(0.0, 0.0);

out vec3 outcolor;

void main() {
    outcolor = color;
    vec2 new_position = local_position * vec2(scale_x, scale_y) + position;
    new_position.y *= screen_ratio;
    // new_position *= zoom;
    gl_Position = vec4((new_position + camera_position) * zoom, 0.0, 1.0);
}