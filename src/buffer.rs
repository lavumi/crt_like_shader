use bytemuck::{Pod, Zeroable};
use wgpu::*;
use util::DeviceExt;
use crate::config::*;


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
    pub uv: [f32; 4],
    pub model: [[f32; 4]; 4],
    pub color: [f32;3],
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
                    shader_location: 3,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as BufferAddress,
                    shader_location: 4,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as BufferAddress,
                    shader_location: 5,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as BufferAddress,
                    shader_location: 6,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as BufferAddress,
                    shader_location: 7,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 20]>() as BufferAddress,
                    shader_location: 8,
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}



pub struct TileRenderData {
    pub char: u8,
    pub position: [usize; 2],
    pub color : [f32;3]
}

impl TileRenderData {
    pub fn get_instance_matrix(&self) -> InstanceTileRaw {
        let char_x = self.char % 16;
        let char_y = self.char / 16;

        let uv = [
            (char_x) as f32 * CHR_UV,
            (char_x+1) as f32 * CHR_UV,
            (char_y) as f32 * CHR_UV,
            (char_y+1) as f32 * CHR_UV,
        ];
        let model = cgmath::Matrix4::from_translation(cgmath::Vector3 {
            x: self.position[0] as f32 / SCREEN_COLS as f32 * 2.0 - 1.0,
            y: self.position[1] as f32 / SCREEN_ROWS as f32 * - 2.0 + 1.0,
            z: 0.0
        }).into();
        let color = self.color;
        InstanceTileRaw {
            uv,
            model,
            color
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

    pub fn new(device : &Device, size : &[f32;2]) -> Self {
        let x_size = size[0];//2.0 / SCREEN_COLS as f32;
        let y_size = size[1];//2.0 / SCREEN_ROWS as f32;
        //region [ Vertex Data ]
        let vertex: [Vertex; 4] = [
            //Front
            Vertex {
                position: [0., -y_size, 0.0],
                tex_coords: [1.0, 0.0],
                // tex_coords: [offset[0] , offset[1] + uv_size[1]],
            },
            Vertex {
                position: [x_size, -y_size, 0.0],
                tex_coords: [0.0, 0.],
                // tex_coords: [offset[0] +uv_size[0], offset[1] +uv_size[1]],
            },
            Vertex {
                position: [x_size, 0., 0.0],
                tex_coords: [0.0, 1.0],
                // tex_coords: [offset[0] +uv_size[0], offset[1] +0.0],
            },
            Vertex {
                position: [0., 0., 0.0],
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
            &util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertex),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            }
        );

        let index_buffer = device.create_buffer_init(
            &util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            }
        );

        let num_indices = indices.len() as u32;

        let instances:Vec<InstanceTileRaw> = Vec::new();
        let instance_buffer = device.create_buffer_init(
            &util::BufferInitDescriptor {
                label: Some(format!("Instance Buffer").as_str()),
                contents: bytemuck::cast_slice(&instances),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            }
        );
        let num_instances = instances.len() as u32;

        Mesh {
            vertex_buffer,
            index_buffer,
            instance_buffer,
            num_indices,
            num_instances,
        }
    }



    pub fn replace_instance(&mut self, buffer: Buffer , num_instance : u32){
        self.instance_buffer = buffer;
        self.num_instances = num_instance;
    }
}


