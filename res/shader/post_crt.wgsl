struct VertexInput {
    @location(0) position: vec3<f32>,
//    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) position: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    var position = model.position + vec3<f32>(-1.0,1.0,0.0);
    out.tex_coords = vec2<f32>(
        model.position[0] * 0.5,
        - model.position[1] * 0.5,
    ) ;
    out.clip_position = vec4<f32>(position, 1.0);
    out.position = position;
    return out;
}


@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;


@group(1) @binding(0)
var<uniform> time: vec4<f32>;





@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//       return vec4<f32> (in.tex_coords, 0.0,1.0);


    let texture = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    let alpha_threshold : f32 = 0.5;
    let tv_border = add_tv_border(in.position.xy);
    let factor = add_scan_line( in.position.y );
    let color = add_noise(in.position , texture.rgb);

    return vec4<f32>( color * factor, 1.0) + tv_border;
}

fn add_noise(modelPos : vec3<f32>, color : vec3<f32>) -> vec3<f32>{
    let mul_model_pos = modelPos * 100.0;
    let t = time[0] * 4.0;
    let factor1 = 1.0 - time_noise(mul_model_pos, t) * 0.25;
    let baseColor = vec3(
      time_noise(mul_model_pos, t),
      time_noise(mul_model_pos, t * 2.0),
      time_noise(mul_model_pos, t * 3.0));

    return baseColor * 0.025 + color * factor1;
}

fn add_scan_line( modelPosY: f32)-> f32 {
    // Add 10 so we don't have to deal with negative numbers.
    let t :f32= 10.0 + modelPosY * 24.0 * 8.0;

    let distToFloor = fract(t);
    let distToCeil = 1.0 - distToFloor;
    let distToNearestInt = min(distToFloor, distToCeil);

    // Integers are the black bands, so we want the scanline intensity to
    // be 1 there, and quickly fall towards 0 as we get away from them:
    let intensity = 1.0 - smoothstep(0.0, 0.3, distToNearestInt);
    let factor = max(0.0, 1.0 - intensity * 0.7);
    return factor;
}

fn add_tv_border( modelPos : vec2<f32>) -> vec4<f32> {
    let distToBorderH = abs(abs(modelPos.x) - 1.0);
    let distToBorderV = abs(abs(modelPos.y) - 1.0);
    let distToBorder = min(distToBorderH, distToBorderV);
    let f = 1.0 -smoothstep(0.0, 0.2, distToBorder);
    return vec4(f, f, f, 1.0) * 0.02;
}



// Original Source:
// https://github.com/ashima/webgl-noise/blob/master/src/noise3D.glsl
// (MIT licensed)
//
// Description : Array and textureless GLSL 2D/3D/4D simplex
//               noise functions.
//      Author : Ian McEwan, Ashima Arts.
//  Maintainer : stegu
//     Lastmod : 20201014 (stegu)
//     License : Copyright (C) 2011 Ashima Arts. All rights reserved.
//               Distributed under the MIT License. See LICENSE file.
//               https://github.com/ashima/webgl-noise
//               https://github.com/stegu/webgl-noise
//
// Convert to wgsl
fn mod289( x:vec3<f32>)->vec3<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn mod289_vec4(x:vec4<f32>)->vec4<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn permute(x:vec4<f32>)->vec4<f32> {
    return mod289_vec4(((x*34.0)+10.0)*x);
}

fn taylorInvSqrt(r:vec4<f32>)->vec4<f32> {
    return 1.79284291400159 - 0.85373472095314 * r;
}

fn time_noise(modelPos: vec3<f32> , time : f32) -> f32 {
    let v = vec3<f32>(modelPos.xy, time);

    let C = vec2<f32>(0.16666666666,0.33333333333);
    let D = vec4<f32>(0.0, 0.5, 1.0, 2.0);

    //First corner
    var i = floor(v + dot(v, C.yyy));
    let x0 = v - i + dot(i, C.xxx);


    //Other corners
    let g = step(x0.yzx, x0.xyz);
    let l = 1.0 - g;
    let i1 = min( g.xyz, l.zxy );
    let i2 = max( g.xyz, l.zxy );

    //   x0 = x0 - 0.0 + 0.0 * C.xxx;
    //   x1 = x0 - i1  + 1.0 * C.xxx;
    //   x2 = x0 - i2  + 2.0 * C.xxx;
    //   x3 = x0 - 1.0 + 3.0 * C.xxx;
    let x1 = x0 - i1 + C.xxx;
    let x2 = x0 - i2 + C.yyy; // 2.0*C.x = 1/3 = C.y
    let x3 = x0 - D.yyy;      // -1.0+3.0*C.x = -0.5 = -D.y

    // Permutations
    i = mod289(i);
    let p = permute( permute( permute(
         i.z + vec4<f32>(0.0, i1.z, i2.z, 1.0 ))
        + i.y + vec4<f32>(0.0, i1.y, i2.y, 1.0 ))
        + i.x + vec4<f32>(0.0, i1.x, i2.x, 1.0 ));


    // Gradients: 7x7 points over a square, mapped onto an octahedron.
    // The ring size 17*17 = 289 is close to a multiple of 49 (49*6 = 294)
    let n_ = 0.142857142857; // 1.0/7.0
    let  ns = n_ * D.wyz - D.xzx;

    let j = p - 49.0 * floor(p * ns.z * ns.z);  //  mod(p,7*7)

    let x_ = floor(j * ns.z);
    let y_ = floor(j - 7.0 * x_ );    // mod(j,N)

    let x = x_ *ns.x + ns.yyyy;
    let y = y_ *ns.x + ns.yyyy;
    let h = 1.0 - abs(x) - abs(y);

    let b0 = vec4<f32>( x.xy, y.xy );
    let b1 = vec4<f32>( x.zw, y.zw );

    //vec4 s0 = vec4(lessThan(b0,0.0))*2.0 - 1.0;
    //vec4 s1 = vec4(lessThan(b1,0.0))*2.0 - 1.0;
    let s0 = floor(b0)*2.0 + 1.0;
    let s1 = floor(b1)*2.0 + 1.0;
    let sh = -step(h, vec4<f32>(0.0));

    let a0 = b0.xzyw + s0.xzyw*sh.xxyy ;
    let a1 = b1.xzyw + s1.xzyw*sh.zzww ;

    var p0 = vec3(a0.xy,h.x);
    var p1 = vec3(a0.zw,h.y);
    var p2 = vec3(a1.xy,h.z);
    var p3 = vec3(a1.zw,h.w);


    //Normalise gradients
      let norm = taylorInvSqrt(vec4<f32>(dot(p0,p0), dot(p1,p1), dot(p2, p2), dot(p3,p3)));
      p0 *= norm.x;
      p1 *= norm.y;
      p2 *= norm.z;
      p3 *= norm.w;

    // Mix final noise value
      var m = max(vec4<f32>(0.5) - vec4<f32>(dot(x0,x0), dot(x1,x1), dot(x2,x2), dot(x3,x3)), vec4<f32>(0.0));
      m = m * m;

      return 105.0 * dot( m*m, vec4<f32>(dot(p0,x0), dot(p1,x1),dot(p2,x2), dot(p3,x3)));
}



//const BLURSCALEX : f32 = 2.45;
//const LOWLUMSCAN : f32 =  10.0;
//const HILUMSCAN : f32 =  10.0;
//const BRIGHTBOOST : f32 =  0.25;
//const MASK_DARK : f32 =  0.25;
//const MASK_FADE : f32 =  0.8;

//@fragment
//fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//
//    let texture_size = vec2<f32>(800.0,600.0);
//    let output_size = vec2<f32>(800.0,600.0);
//    let input_size = vec2<f32>(800.0,600.0);
//
//    let ratio = texture_size.x / input_size.x;
//
//    let inv_dims = 1.0 / texture_size;
//
//
//    let p0 = in.tex_coords * texture_size;
//    let i = floor(p0) + 0.50;
//    let f = p0 - i;
//    var p = (i + 4.0*f*f*f) * inv_dims;
//    p.x = mix( p.x , in.tex_coords.x, BLURSCALEX);
//
//    let Y = f.y*f.y;
//    let YY = Y*Y;
//
//    let whichmask = floor(in.tex_coords.x * output_size.x * ratio)*-0.5;
//    let mask = 1.0 + f32(fract(whichmask) < 0.5) * -MASK_DARK;
//
//
//    let colour = vec3<f32>(1.0,1.0,1.0);//textureSample(t_diffuse, s_diffuse, in.tex_coords).rgb;
//
//    let scanLineWeight = (BRIGHTBOOST - LOWLUMSCAN*(Y - 2.05*YY));
//    let scanLineWeightB = 1.0 - HILUMSCAN*(YY - 2.8 * YY * Y);
//
//    return vec4<f32>(colour.rgb * mix(scanLineWeight*mask, scanLineWeightB, dot(colour.rgb,vec3(MASK_FADE * 0.33333))),1.0);
//}