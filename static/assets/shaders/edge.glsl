const vec4 _DepthGradientShallow = vec4(0.325, 0.807, 0.971, 0.725);
const vec4 _DepthGradientDeep = vec4(0.086, 0.207, 0.87, 0.889);

uniform sampler2D surfaceNoise;

const vec3 surface_noise_scroll = vec3(0.00, 0.2, 0.0);

varying vec2 v_uv;

const float SMOOTHSTEP_AA = 0.01;


#ifdef VERTEX_SHADER
attribute vec3 a_v;
attribute vec2 a_uv;

uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;


void main() {
    v_uv = a_uv;
    v_uv.y = a_v.z / (-5.0);
    v_uv.x = a_v.x / 200.0 + 0.5;
    gl_Position = u_projection_matrix * u_view_matrix * vec4(a_v, 1.0);
}
#endif

float rand(vec2 n) { 
	return fract(sin(dot(n, vec2(12.9898, 4.1414))) * 43758.5453);
}

float noise(vec2 p){
	vec2 ip = floor(p);
	vec2 u = fract(p);
	u = u*u*(3.0-2.0*u);
	
	float res = mix(
		mix(rand(ip),rand(ip+vec2(1.0,0.0)),u.x),
		mix(rand(ip+vec2(0.0,1.0)),rand(ip+vec2(1.0,1.0)),u.x),u.y);
	return res*res;
}

vec4 alphaBlend(vec4 top, vec4 bottom)
{
	vec3 color = (top.rgb * top.a) + (bottom.rgb * (1.0 - top.a));
	float alpha = top.a + bottom.a * (1.0 - top.a);
	
	return vec4(color, alpha);
}

#ifdef FRAGMENT_SHADER
uniform float u_time;
void main() {
    float x = sin((v_uv.y - u_time) * 5.0) * 0.5 + 0.5;
    x *= 0.2;
    float streaks = noise(vec2(v_uv.x * 40.0 + sin(u_time) * 2.0, v_uv.y + u_time)) * 0.3;
    float streaks2 = noise(vec2(v_uv.x * 120.0, v_uv.y / 10.0 - u_time)) * 1.4;
    float wavy = abs(sin((v_uv.x + x / 35.0) * 150.0)) * 0.3 + 0.1;
    gl_FragColor = mix(_DepthGradientDeep, _DepthGradientShallow, clamp(streaks + wavy + x, 0.3, 0.5));

    // foam
    float surfaceNoiseCutoff = clamp(v_uv.y + 0.1, 0.0, 1.0);
	
	vec2 noise_uv = vec2(
		(v_uv.x * 10.0 + u_time * surface_noise_scroll.x), 
		(v_uv.y / 2.0 - u_time * surface_noise_scroll.y)
	);
	float surfaceNoiseSample = texture2D(surfaceNoise, noise_uv).r;
	float surfaceNoiseAmount = smoothstep(surfaceNoiseCutoff - SMOOTHSTEP_AA, surfaceNoiseCutoff + SMOOTHSTEP_AA, surfaceNoiseSample);
    gl_FragColor = mix(gl_FragColor, vec4(1.0), surfaceNoiseAmount);

    // streaks
    gl_FragColor = mix(gl_FragColor, vec4(1.0), clamp(pow(streaks2, 50.0), 0.0, 0.9));
    // gl_FragColor = vec4(surfaceNoiseSample);
}
#endif