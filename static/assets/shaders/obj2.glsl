varying vec2 v_uv;

#ifdef VERTEX_SHADER
attribute vec3 a_v;
attribute vec2 a_uv;

uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_model_matrix;

void main() {
    v_uv = a_uv;
    gl_Position = u_projection_matrix * u_view_matrix * u_model_matrix * vec4(a_v, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
void main() {
    gl_FragColor = pack4(gl_FragCoord.z);
}
#endif