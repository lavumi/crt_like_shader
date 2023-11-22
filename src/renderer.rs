use std::iter;
use instant::Instant;
use std::sync::Arc;
use image::GenericImageView;
use winit::window::Window;
use wgpu::*;
use wgpu::util::DeviceExt;
use crate::buffer::*;
use winit::dpi::PhysicalSize;
use crate::config::*;



pub struct Renderer {
    pub device: Device,
    surface: Surface,

    pub queue: Queue,
    pub config: SurfaceConfiguration,
    screen_buffer : [Tile; SCREEN_COLS * SCREEN_ROWS],


    main_view : TextureView,

    camera_bind_group: BindGroup,

    texture_bind_group_layout: BindGroupLayout,
    time_bind_group:BindGroup,
    time_buffer:Buffer,

    diffuse_render_pipeline: RenderPipeline,
    post_render_pipeline: RenderPipeline,



    sampler: Sampler,
    bind_group: Option<Arc<BindGroup>>,
    post_process_bind_group : BindGroup,

    mesh: Mesh,
    screen_mesh: Mesh,

    init_time : Instant,

}

impl Renderer {
    pub async fn new(window: &Window, game_config: &GameConfig) -> Self {
        let size = PhysicalSize::new(game_config.options.screen_size[0] * 2, game_config.options.screen_size[1] * 2);
        let instance = Instance::new(InstanceDescriptor::default());
        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    features: Features::empty(),
                    // WebGL doesn't support all of wgpu`s features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        Limits::downlevel_webgl2_defaults()
                    } else {
                        Limits::default()
                    },
                },
                // Some(&std::path::Path::new("trace")), // Trace path
                None,
            )
            .await
            .unwrap();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            // .filter(|f| f.describe().srgb)
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        // let viewport_data = [0., 0., size.width as f32, size.height as f32, 0., 1.];




        //region [ Main Render Path ]
        let camera_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor{
            label: Some("camera_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry{
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        });
        let camera_matrix: [[f32; 4]; 4] = Renderer::make_camera_view().into();
        let camera_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_matrix]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
        });

        let texture_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let time_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor{
            label: Some("time_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry{
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        });
        let time_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
                label: Some("Time Buffer"),
                contents: bytemuck::cast_slice(&[0.5, 0.0, 0.0, 0.0]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });
        let time_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("time_bind_group"),
            layout: &time_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: time_buffer.as_entire_binding(),
                }
            ],
        });

        let shader = device.create_shader_module(include_wgsl!("../res/shader/texture.wgsl"));
        let diffuse_render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &texture_bind_group_layout,
                &time_bind_group_layout
            ],
            push_constant_ranges: &[],
        });
        let diffuse_render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Base Render Pipeline"),
            layout: Some(&diffuse_render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &vec![Vertex::desc(), InstanceTileRaw::desc()],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: true,
            },
            multiview: None,
        });

        //endregion

        //region [ Post Render Path ]
        let crt_shader = device.create_shader_module(include_wgsl!("../res/shader/crt.wgsl"));
        let post_render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout
            ],
            push_constant_ranges: &[],
        });
        let post_render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Post Render Pipeline"),
            layout: Some(&post_render_pipeline_layout),
            vertex: VertexState {
                module: &crt_shader,
                entry_point: "vs_main",
                buffers: &vec![Vertex::desc()],
            },
            fragment: Some(FragmentState {
                module: &crt_shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: true,
            },
            multiview: None,
        });
        let main_texture = device.create_texture(&TextureDescriptor {
            label: Some("Main render texture"),
            size : Extent3d{
                width : size.width,
                height : size.height,
                depth_or_array_layers:1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: surface_format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let main_view = main_texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Linear,
            ..Default::default()
        });
        let post_process_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&main_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                }
            ],
            label: Some("diffuse_bind_group"),
        });

        //endregion

        let tile_size = [
            2.0 / SCREEN_COLS as f32,
            2.0 / SCREEN_ROWS as f32
        ];
        let mesh = Mesh::new(&device, &tile_size);



        let screen_mesh = Mesh::new(&device, &[2.0,2.0]);
        let mut screen_buffer = [ Tile::default(); SCREEN_ROWS * SCREEN_COLS];
        let map = game_config.get_map();
        for (index, tile) in map.iter().enumerate(){
            screen_buffer[index].char = tile.char;
            screen_buffer[index].color = tile.color;
        }

        let init_time = Instant::now();


        Self {
            device,
            surface,
            queue,
            config,
            mesh,
            screen_mesh,
            diffuse_render_pipeline,
            time_bind_group,
            time_buffer,
            texture_bind_group_layout,
            bind_group: None,
            sampler,
            screen_buffer,
            post_process_bind_group,
            post_render_pipeline,
            main_view,
            init_time,
            camera_bind_group
        }
    }



    fn make_camera_view()-> cgmath::Matrix4<f32> {
        let eye : cgmath::Point3<f32> =(0.0, 0.0, 3.0).into();
        let target: cgmath::Point3<f32>=(0.0, 0.0, 0.0).into();
        let up: cgmath::Vector3<f32>=cgmath::Vector3::unit_y();
        let aspect:f32 = 1.0;
        let fov_y:f32 = 45.0;
        let z_near:f32 = 0.1;
        let z_far:f32 = 100.0;

        let view = cgmath::Matrix4::look_at_rh(eye, target, up);
        let proj = cgmath::perspective(cgmath::Deg(fov_y), aspect, z_near, z_far);
        OPENGL_TO_WGPU_MATRIX * proj * view
    }

    pub fn set_texture(&mut self, bytes: &[u8]) {
        // let img = image
        let img = image::load_from_memory(bytes).unwrap();
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();
        let size = Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let format = TextureFormat::Rgba8UnormSrgb;
        let texture = self.device.create_texture(&TextureDescriptor {
            label: Some("texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.queue.write_texture(
            ImageCopyTexture {
                aspect: TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
            },
            &rgba,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Option::from(4 * dimensions.0),
                rows_per_image: Option::from(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&TextureViewDescriptor::default());


        let diffuse_bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.sampler),
                }
            ],
            label: Some("diffuse_bind_group"),
        });

        self.bind_group = Some(Arc::from(diffuse_bind_group));
    }
    pub fn init_instances(&mut self){
        let instances = self.screen_buffer.iter().enumerate().map(|(i, &tile)|{
            TileRenderData{
                char : tile.char,
                position: [i  % SCREEN_COLS, i  / SCREEN_COLS],
                color : tile.color
            }.get_instance_matrix()
        }).collect::<Vec<_>>();


        let instance_buffer = self.device.create_buffer_init(
            &util::BufferInitDescriptor {
                label: Some(format!("Instance Buffer").as_str()),
                contents: bytemuck::cast_slice(&instances),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            }
        );
        let num_instances = instances.len() as u32;
        self.mesh.replace_instance(instance_buffer, num_instances);
    }
    pub fn render(&self) -> Result<(), SurfaceError> {

        let render_time = Instant::now().duration_since(self.init_time);
        let time_data:[f32;4] = [render_time.as_millis() as f32, 0.0, 0.0, 0.0];
        self.queue.write_buffer(&self.time_buffer, 0, bytemuck::cast_slice(&[time_data]));

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());


        let mut encoder = self.device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &self.main_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.diffuse_render_pipeline);

            match &self.bind_group {
                None => {}
                Some(bg) => {
                    render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                    render_pass.set_bind_group(1, bg, &[]);
                    render_pass.set_bind_group(2, &self.time_bind_group , &[]);
                    render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
                    render_pass.set_vertex_buffer(1, self.mesh.instance_buffer.slice(..));
                    render_pass.set_index_buffer(self.mesh.index_buffer.slice(..), IndexFormat::Uint16);
                    render_pass.draw_indexed(0..self.mesh.num_indices, 0, 0..self.mesh.num_instances);
                }
            }
        }

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Crt Post Process Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.post_render_pipeline);

            render_pass.set_bind_group(0, &self.post_process_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.screen_mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.screen_mesh.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.screen_mesh.num_indices, 0, 0..1);


        }


        self.queue.submit(iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);