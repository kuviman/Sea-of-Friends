varying vec2 v_uv;

#ifdef VERTEX_SHADER
attribute vec3 a_v;
attribute vec2 a_uv;

uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;

void main() {
    v_uv = a_uv;
    v_uv.y = a_v.z / (-5.0);
    gl_Position = u_projection_matrix * u_view_matrix * vec4(a_v, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform float u_time;
void main() {
    float x = sin((v_uv.y - u_time) * 10.0) * 0.5 + 0.5;
    x *= 0.2;
    gl_FragColor = vec4(x, x, 0.8 + x, 1.0);
}
#endif