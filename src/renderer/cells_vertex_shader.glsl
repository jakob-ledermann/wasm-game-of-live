attribute vec2 position;
attribute lowp float alive;

uniform ivec2 size;

varying lowp float frag_alive;

void main() {
    vec2 float_position;
    float_position = vec2(position.x / float(size.x) * 2.0, position.y / float(size.y) * 2.0);
    float_position = float_position - vec2(1, 1);
    frag_alive = alive;
    gl_Position = vec4(float_position.x, -float_position.y, 1.0, 1.0);
}
