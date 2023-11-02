use bytemuck::{Pod, Zeroable};
use wgpu::*;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub(crate) position: [f32; 3],
    pub(crate) tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [VertexAttribute; 2] =
        vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    pub fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct InstanceTileRaw {
    pub(crate) uv: [f32; 4],
    pub(crate) model: [[f32; 4]; 4],
}
impl InstanceTileRaw {
    pub fn desc<'a>() -> VertexBufferLayout<'a> {
        use std::mem;
        VertexBufferLayout {
            array_stride: mem::size_of::<InstanceTileRaw>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 4,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as BufferAddress,
                    shader_location: 5,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as BufferAddress,
                    shader_location: 6,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as BufferAddress,
                    shader_location: 7,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as BufferAddress,
                    shader_location: 8,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }
}



pub struct TileRenderData {
    pub uv: [f32; 4],
    pub position: [f32; 3],
    pub size: [f32; 2],
}

impl TileRenderData {
    pub fn get_instance_matrix(&self) -> InstanceTileRaw {
        let position = cgmath::Vector3 { x: self.position[0], y: self.position[1], z: self.position[2] };
        let translation_matrix = cgmath::Matrix4::from_translation(position);
        let scale_matrix = cgmath::Matrix4::from_nonuniform_scale(self.size[0], self.size[1], 1.0);
        let model = (translation_matrix * scale_matrix).into();

        InstanceTileRaw {
            uv: self.uv,
            model,
        }

    }
}


pub struct Mesh {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub instance_buffer: Buffer,
    pub num_indices: u32,
    pub num_instances: u32,
}
impl Mesh {

    pub fn new(device : &Device) -> Self {
        //region [ Vertex Data ]
        let tile_size = [1.0, 1.0];
        let tile_size_half = [tile_size[0] * 0.5, tile_size[1] * 0.5];
        let vertex: [Vertex; 4] = [
            //Front
            Vertex {
                position: [-tile_size_half[0], -tile_size_half[1], 0.0],
                tex_coords: [1.0, 0.0],
                // tex_coords: [offset[0] , offset[1] + uv_size[1]],
            },
            Vertex {
                position: [tile_size_half[0], -tile_size_half[1], 0.0],
                tex_coords: [0.0, 0.],
                // tex_coords: [offset[0] +uv_size[0], offset[1] +uv_size[1]],
            },
            Vertex {
                position: [tile_size_half[0], tile_size_half[1], 0.0],
                tex_coords: [0.0, 1.0],
                // tex_coords: [offset[0] +uv_size[0], offset[1] +0.0],
            },
            Vertex {
                position: [-tile_size_half[0], tile_size_half[1], 0.0],
                tex_coords: [1.0, 1.0],
                // tex_coords: offset ,
            }
        ];
        let indices: [u16; 6] = [
            //front
            0, 1, 2,
            2, 3, 0,
        ];

        //endregion

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertex),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        let num_indices = indices.len() as u32;



        let instance_tile_raw = TileRenderData{
            uv:  [0.0, 1.0, 0.0, 1.0],
            position: [0.,0.,0.],
            size: [1., 1.],
        }.get_instance_matrix();

        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(format!("Instance Buffer").as_str()),
                contents: bytemuck::cast_slice(&[instance_tile_raw]),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );
        let num_instances =1;//instance_data.len() as u32;

        Mesh {
            vertex_buffer,
            index_buffer,
            instance_buffer,
            num_indices,
            num_instances,
        }
    }
    pub fn replace_instance(&mut self, buffer: wgpu::Buffer , num_instance : u32){
        self.instance_buffer = buffer;
        self.num_instances = num_instance;
    }
}


