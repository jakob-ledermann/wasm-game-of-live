    attribute vec2 position;

    uniform ivec2 size;

    void main() {
        vec2 float_position;
        float_position = vec2(position.x / float(size.x) * 2.0, position.y / float(size.y) * 2.0);
        float_position = float_position - vec2(1, 1);
        gl_Position = vec4(float_position.x, -float_position.y, 1.0, 1.0);
    }
