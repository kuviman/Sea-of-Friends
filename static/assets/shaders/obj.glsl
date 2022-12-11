varying vec2 v_uv;
varying float v_light;

#ifdef VERTEX_SHADER
attribute vec3 a_v;
attribute vec2 a_uv;
attribute vec3 a_vn;

uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
attribute mat4 i_model_matrix;

void main() {
    v_uv = a_uv;
    v_light = dot(mat3(i_model_matrix) * a_vn, normalize(vec3(1.0, 2.0, 3.0)));
    v_light = max(v_light, 0.0) * 0.2 + 0.8;
    gl_Position = u_projection_matrix * u_view_matrix * i_model_matrix * vec4(a_v, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;
uniform vec4 u_color;
void main() {
    gl_FragColor = texture2D(u_texture, v_uv) * u_color;
    gl_FragColor.xyz *= v_light;
    if (gl_FragColor.w < 0.5) {
        discard;
    }
}
#endif