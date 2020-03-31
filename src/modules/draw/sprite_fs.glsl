uniform float depth;
uniform sampler2D image;
// uniform sampler2DShadow depth_buffer;
// uniform bool use_depth_buffer;
uniform vec4 brighten;
uniform vec4 darken;
uniform float desaturation;

in vec2 v_uv;

out vec4 fragment;

void main() {
    fragment = texture(image, v_uv);

    float average = (fragment.r + fragment.g + fragment.b) / 3.0;
    fragment.rgb += desaturation * (average - fragment.rgb);

    fragment += brighten * (1.0 - fragment);
    fragment *= darken;

    float d = depth * 2;
    // gl_FragDepth = depth;

    // if (use_depth_buffer) {
    //     gl_FragDepth = texture(depth_buffer, vec3(v_uv, 0));
    // } else {
    //     gl_FragDepth = depth;
    // }
}
