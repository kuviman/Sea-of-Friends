varying vec2 v_uv;

#ifdef VERTEX_SHADER
attribute vec3 a_v;
attribute vec2 a_uv;

uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
attribute mat4 i_model_matrix;

void main() {
    v_uv = a_uv;
    gl_Position = u_projection_matrix * u_view_matrix * i_model_matrix * vec4(a_v, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;
uniform vec4 u_color;
void main() {
    gl_FragColor = texture2D(u_texture, v_uv) * u_color;
    if (gl_FragColor.w < 0.5) {
        discard;
    }
}
#endif