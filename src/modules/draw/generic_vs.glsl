const vec2[] QUAD_POSITIONS = vec2[](
    vec2(0.0, 0.0),
    vec2(1.0, 0.0),
    vec2(1.0, 1.0),
    vec2(0.0, 1.0)
);

uniform vec2 screen_size;
uniform float hidpi_factor;
uniform vec2 image_size;
uniform vec2 subimage_offset;
uniform vec2 subimage_size;

uniform float depth;
uniform vec2 origin;
uniform vec2 position;
uniform float rotation;
uniform vec2 scale;

out vec2 v_uv;

vec2 rotate(vec2 in_vec) {
    return vec2(
        in_vec.x * cos(rotation) - in_vec.y * sin(rotation),
        in_vec.x * sin(rotation) + in_vec.y * cos(rotation)
    );
}

void main() {
    vec2 xy = QUAD_POSITIONS[gl_VertexID];
    vec2 uv_ratio = hidpi_factor * subimage_size / screen_size;

    vec2 object_position = hidpi_factor * position / screen_size;
    vec2 vertex_offset = scale
        * uv_ratio
        * rotate((xy - origin) * subimage_size)
        / subimage_size;
    gl_Position = vec4(object_position + vertex_offset, 0.0, 1.0);

    vec2 adjusted_offset = subimage_offset / image_size;
    vec2 adjusted_size = subimage_size / image_size;

    v_uv = xy * 2.0 - 1.0;
    v_uv = vec2(v_uv.x, -v_uv.y);
    v_uv = v_uv * 0.5 + 0.5;
    v_uv = (adjusted_offset + adjusted_size * v_uv);
    v_uv = vec2(v_uv.x, 1.0 - v_uv.y);
}
