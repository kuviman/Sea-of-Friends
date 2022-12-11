varying vec2 v_uv;
varying vec3 v_v;
varying float v_height;
varying mat4 v_model_matrix;
// varying vec4 v_color;

#ifdef VERTEX_SHADER
attribute vec3 a_v;
attribute vec2 a_uv;

attribute mat4 i_model_matrix;
attribute float i_height;
// attribute vec4 i_color;

uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;

void main() {
    v_height = i_height;
    v_model_matrix = i_model_matrix;
    // v_color = i_color;
    v_uv = a_uv;
    v_v = a_v;
    gl_Position = u_projection_matrix * u_view_matrix * i_model_matrix * vec4(a_v, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;
void main() {
    gl_FragColor = texture2D(u_texture, v_uv);
    if (gl_FragColor.w < 0.5) {
        discard;
    }
    if (v_height < 0.0 && distance(vec3(0.0), v_model_matrix[3].xyz) < 100.0) {
        gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
    }
}
#endif