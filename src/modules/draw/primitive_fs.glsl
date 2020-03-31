const float PI = 355.0 / 113.0;
const int ARC_ID = 0;
const int CIRCLE_ID = 1;
const int LINE_ID = 2;
const int RECTANGLE_ID = 3;

uniform float depth;
uniform vec2 image_size;

uniform int shape_id;

uniform vec4 color;
uniform float radius;
uniform float thickness;

uniform float arc_from;
uniform float arc_to;

uniform vec2 line_from;
uniform vec2 line_to;

in vec2 v_uv;

out vec4 fragment;

vec2 pos() {
    return image_size * (v_uv - 0.5);
}

void draw_arc() {
    vec2 p = pos();
    float angle = atan(p.y, p.x);
    while (angle < 0.0) {
        angle += 2.0 * PI;
    }
    float distance_from_center = distance(vec2(0.0, 0.0), pos());

    bool within_angle = arc_from <= arc_to
        ? angle >= arc_from && angle <= arc_to
        : angle >= arc_from || angle <= arc_to;
    within_angle = within_angle || arc_from == arc_to;
    bool within_distance = distance_from_center >= radius - thickness
        && distance_from_center <= radius;

    if (within_angle && within_distance) {
        fragment = color;
    } else {
        fragment = vec4(0.0, 0.0, 0.0, 0.0);
    }
}

void draw_circle() {
    bool within_distance = distance(vec2(0.0, 0.0), pos()) <= radius;
    if (within_distance) {
        fragment = color;
    } else {
        fragment = vec4(0.0, 0.0, 0.0, 0.0);
    }
}

void draw_line() {
    vec2 me = pos();
    float numerator = (line_to.y - line_from.y) * me.x;
    numerator -= (line_to.x - line_from.x) * me.y;
    numerator += line_to.x * line_from.y;
    numerator -= line_to.y * line_from.x;
    numerator = abs(numerator);
    float denominator = sqrt(pow(line_to.y - line_from.y, 2) + pow(line_to.x - line_from.x, 2));
    float distance_line_from_line = numerator / denominator;

    if (distance_line_from_line <= thickness / 2.0) {
        fragment = color;
    } else {
        fragment = vec4(0.0, 0.0, 0.0, 0.0);
    }
}

void draw_rectangle() {
    fragment = color;
}

void main() {
    switch (shape_id) {
        case ARC_ID:
            draw_arc();
            break;
        case CIRCLE_ID:
            draw_circle();
            break;
        case LINE_ID:
            draw_line();
            break;
        case RECTANGLE_ID:
            draw_rectangle();
            break;
        default:
            fragment = vec4(0.0, 0.0, 0.0, 0.0);
    }

    float d = depth * 2;
}
