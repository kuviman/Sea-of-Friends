/**
* Ported from the original unity shader by Erik Roystan Ross
* https://roystan.net/articles/toon-water.html
* https://github.com/IronWarrior/ToonWaterShader
* Camera Depth taken from Bastiaan Olij's video on: https://www.youtube.com/watch?v=Jq3he9Lbj7M
*/

const float SMOOTHSTEP_AA = 0.01;

uniform sampler2D surfaceNoise;
uniform sampler2D distortNoise;
uniform float u_time;

const float beer_factor = 0.8;

const float foam_distance = 0.01;
const float foam_max_distance = 0.4;
const float foam_min_distance = 0.04;
const vec4 foam_color  = vec4(1.0);

const vec2 surface_noise_tiling = vec2(1.0, 4.0);
const vec3 surface_noise_scroll = vec3(0.03, 0.03, 0.0);
const float surface_noise_cutoff = 0.9;// 0.777;
const float surface_distortion_amount = 0.27;

const vec4 _DepthGradientShallow = vec4(0.325, 0.807, 0.971, 0.725);
const vec4 _DepthGradientDeep = vec4(0.086, 0.407, 1, 0.749);
const float _DepthMaxDistance = 1.0;
const float _DepthFactor = 1.0;

uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_model_matrix;

varying vec2 noiseUV;
varying vec2 distortUV;
varying vec3 viewNormal;

vec4 alphaBlend(vec4 top, vec4 bottom)
{
	vec3 color = (top.rgb * top.a) + (bottom.rgb * (1.0 - top.a));
	float alpha = top.a + bottom.a * (1.0 - top.a);
	
	return vec4(color, alpha);
}

varying vec3 v_v;
varying vec4 v_eye_pos;

#ifdef VERTEX_SHADER
attribute vec2 a_uv;
attribute vec3 a_v;
void main() {
    v_v = a_v;
	viewNormal = (u_view_matrix * vec4(0.0, 0.0, 1.0, 0.0)).xyz;
    // (MODELVIEW_MATRIX * vec4(NORMAL, 0.0)).xyz;
	noiseUV = a_uv * surface_noise_tiling;
	distortUV = a_uv;
    v_eye_pos = u_view_matrix * u_model_matrix * vec4(a_v, 1.0);
    gl_Position = u_projection_matrix * v_eye_pos;
}
#endif

#ifdef FRAGMENT_SHADER
uniform vec2 u_framebuffer_size;
uniform sampler2D u_depth_texture;
void main(){
	// https://www.youtube.com/watch?v=Jq3he9Lbj7M
	float depth = unpack4(texture2D(u_depth_texture, gl_FragCoord.xy / u_framebuffer_size));
	depth = depth * 2.0 - 1.0;
	depth = u_projection_matrix[3][2] / (depth + u_projection_matrix[2][2]);
	depth = depth + v_eye_pos.z;
	depth = exp(-depth * beer_factor);
	depth = 1.0 - depth;
	
	// Still unsure how to get properly the NORMAL from the camera
	// this was my best attempt
	vec3 existingNormal = vec3(dFdx(depth), dFdy(depth), 0);
	
	float normalDot = clamp(dot(existingNormal.xyz, viewNormal), 0.0, 1.0);
	float foamDistance = mix(foam_max_distance, foam_min_distance, normalDot);
	
	float foamDepth = clamp(depth / foamDistance, 0.0, 1.0);
	float surfaceNoiseCutoff = foamDepth * surface_noise_cutoff;
	
	vec4 distortNoiseSample = texture2D(distortNoise, distortUV);
	vec2 distortAmount = (distortNoiseSample.xy * 2.0 -1.0) * surface_distortion_amount;
	
	vec2 noise_uv = vec2(
		(noiseUV.x + u_time * surface_noise_scroll.x) + distortAmount.x , 
		(noiseUV.y + u_time * surface_noise_scroll.y + distortAmount.y)
	);
	float surfaceNoiseSample = texture2D(surfaceNoise, noise_uv).r;
	float surfaceNoiseAmount = smoothstep(surfaceNoiseCutoff - SMOOTHSTEP_AA, surfaceNoiseCutoff + SMOOTHSTEP_AA, surfaceNoiseSample);
	
	float waterDepth = clamp(depth / _DepthMaxDistance, 0.0, 1.0) * _DepthFactor;
	vec4 waterColor = mix(_DepthGradientShallow, _DepthGradientDeep, waterDepth);

	vec4 surfaceNoiseColor = foam_color;
    surfaceNoiseColor.a *= surfaceNoiseAmount;
	vec4 color = alphaBlend(surfaceNoiseColor, waterColor);
	
    gl_FragColor = color;
	// ALBEDO = color.rgb;
	// ALPHA = color.a;
}
#endif