varying vec2 v_uv;
varying vec3 v_v;

uniform float height;

#ifdef VERTEX_SHADER
attribute vec3 a_v;
attribute vec2 a_uv;

uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_model_matrix;

void main() {
    v_uv = a_uv;
    v_v = a_v;
    gl_Position = u_projection_matrix * u_view_matrix * u_model_matrix * vec4(a_v, 1.0);
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
    if (height < 0.0) {
        gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
    }
}
#endif