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

    vec2 angle_vector = position_b - position_a;
    float len = length(angle_vector);
    float alpha = atan(angle_vector.y, angle_vector.x);
    vec2 translation = position_a + (position_b - position_a) * 0.5;

    vec2 scaled = local_position * vec2(len, width);
    // rotate scaled by alpha
    vec2 rotated = vec2(scaled.x * cos(alpha) - scaled.y * sin(alpha), scaled.x * sin(alpha) + scaled.y * cos(alpha));
    // translate rotated
    vec2 translated = rotated + translation;

    translated.y *= screen_ratio;
    gl_Position = vec4((translated + camera_position) * zoom, 0.1, 1.0);
}