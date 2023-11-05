struct InstanceInput {
    @location(3) texcoord: vec4<f32>,
    @location(4) matrix_0: vec4<f32>,
    @location(5) matrix_1: vec4<f32>,
    @location(6) matrix_2: vec4<f32>,
    @location(7) matrix_3: vec4<f32>,
    @location(8) color: vec3<f32>,
};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec3<f32>
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {

    let model_matrix = mat4x4<f32>(
        instance.matrix_0,
        instance.matrix_1,
        instance.matrix_2,
        instance.matrix_3,
    );

    var out: VertexOutput;
    out.tex_coords = vec2(
    instance.texcoord[0] * model.tex_coords[0] + instance.texcoord[1] * (1.0-model.tex_coords[0])  ,
    instance.texcoord[2] * model.tex_coords[1] + instance.texcoord[3] * (1.0-model.tex_coords[1])
    );
    out.clip_position =  model_matrix * vec4<f32>(model.position, 1.0);
    out.color = instance.color;
    return out;
}


@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    let alpha_threshold : f32 = 0.0;
    if ( texture.a <= alpha_threshold) {
        discard;
    }

    return vec4<f32>(in.color, 1.0);
}