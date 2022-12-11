varying vec3 v_quad_pos;

#ifdef VERTEX_SHADER
attribute vec3 a_v;

uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_model_matrix;

void main() {
    v_quad_pos = a_v;
    gl_Position = u_projection_matrix * u_view_matrix * u_model_matrix * vec4(a_v, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform vec4 u_color;
uniform float u_lifetime;

void main() {
    gl_FragColor = u_color;

    // Expanding wave
    float len = length(v_quad_pos);
    float width = (1.0 - u_lifetime * u_lifetime) * 0.2;
    float radius = u_lifetime;
    if (len > radius || len < radius - width) {
        discard;
    }
}
#endif