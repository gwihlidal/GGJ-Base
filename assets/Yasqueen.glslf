#version 150 core

in vec2 v_uv;
out vec4 Target0;
uniform sampler2D t_color;

vec2 barrelDistortion(vec2 coord, float amt)
{
	vec2 cc = coord - 0.5;
	float dist = dot(cc, cc);
	return coord + cc * dist * amt;
}

float sat(float t)
{
	return clamp(t, 0.0, 1.0);
}

float linterp(float t)
{
	return sat(1.0 - abs(2.0*t - 1.0));
}

float remap(float t, float a, float b)
{
	return sat((t - a) / (b - a));
}

vec4 spectrum_offset(float t)
{
	// https://github.com/spite/Wagner/blob/master/fragment-shaders/chromatic-aberration-fs.glsl
	vec4 ret;
	float lo = step(t,0.5);
	float hi = 1.0-lo;
	float w = linterp(remap(t, 1.0/6.0, 5.0/6.0));
	ret = vec4(lo,1.0,hi, 1.) * vec4(1.0-w, w, 1.0-w, 1.);
	return pow(ret, vec4(1.0/2.2));
}

const float max_distort = 0.5;
const int num_iter = 11;
const float reci_num_iter_f = 1.0 / float(num_iter);

void main()
{
	vec2 uv=v_uv;//(gl_FragCoord.xy/resolution.xy*.5)+.25;

	vec4 sumcol = vec4(0.0);
	vec4 sumw = vec4(0.0);
	for ( int i=0; i<num_iter;++i )
	{
		float t = float(i) * reci_num_iter_f;
		vec4 w = spectrum_offset(t);
		sumw += w;
		sumcol += w * texture(t_color, barrelDistortion(uv, .6 * max_distort*t));
	}



	Target0 = sumcol / sumw;
/*
	vec4 a1=texture(t_color, barrelDistortion(uv,0.0));
	vec4 a2=texture(t_color, barrelDistortion(uv,0.2));
	vec4 a3=texture(t_color, barrelDistortion(uv,0.4));
	vec4 a4=texture(t_color, barrelDistortion(uv,0.6));

	vec4 a5=texture(t_color, barrelDistortion(uv,0.8));
	vec4 a6=texture(t_color, barrelDistortion(uv,1.0));
	vec4 a7=texture(t_color, barrelDistortion(uv,1.2));
	vec4 a8=texture(t_color, barrelDistortion(uv,1.4));

	vec4 a9=texture(t_color, barrelDistortion(uv,1.6));
	vec4 a10=texture(t_color, barrelDistortion(uv,1.8));
	vec4 a11=texture(t_color, barrelDistortion(uv,2.0));
	vec4 a12=texture(t_color, barrelDistortion(uv,2.2));

	vec4 tx=(a1+a2+a3+a4+a5+a6+a7+a8+a9+a10+a11+a12)/12.;
	Target0 = vec4(tx.rgb, tx.a );*/
}