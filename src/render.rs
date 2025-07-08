// render.rs
use crate::camera::{Camera, CameraUniforms};
use crate::wgpu;
use bytemuck::{Pod, Zeroable};

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Uniforms {
    camera: CameraUniforms,
    width: u32,
    height: u32,
    frame_count: u32,
    _pad: u32,
}

pub struct PathTracer {
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    display_pipeline: wgpu::RenderPipeline,
    display_bind_groups: [wgpu::BindGroup; 2],
}

impl PathTracer {
    pub fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> PathTracer {
        device.on_uncaptured_error(Box::new(|error| {
            panic!("Aborting due to an error: {}", error);
        }));

        let shader_module = compile_shader_module(device);

        let (display_pipeline, display_layout) =
            create_display_pipeline(device, &shader_module, surface_format);

        // Initialize the uniform buffer.
        let uniforms = Uniforms {
            camera: CameraUniforms::zeroed(),
            width,
            height,
            frame_count: 0,
            _pad: 0,
        };

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let radiance_samples = create_sample_texture(&device, width, height);
        // uniform_buffer
        //     .slice(..)
        //     .get_mapped_range_mut()
        //     .copy_from_slice(bytemuck::bytes_of(&uniforms));
        // uniform_buffer.unmap();

        let display_bind_groups = create_display_bind_groups(
            &device,
            &display_layout,
            &radiance_samples,
            &uniform_buffer,
        );

        PathTracer {
            uniforms,
            uniform_buffer,
            display_pipeline,
            display_bind_groups,
        }
    }

    pub fn reset_samples(&mut self) {
        self.uniforms.frame_count = 0;
    }

    pub fn render_frame(
        &mut self,
        camera: &Camera,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
    ) {
        self.uniforms.camera = *camera.uniforms();
        *&mut self.uniforms.frame_count += 1;
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&self.uniforms));
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("render frame"),
        });
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("display pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        render_pass.set_pipeline(&self.display_pipeline);
        // render_pass.set_bind_group(0, &self.display_bind_group, &[]);
        render_pass.set_bind_group(
            0,
            &self.display_bind_groups[(self.uniforms.frame_count % 2) as usize],
            &[],
        );
        render_pass.draw(0..6, 0..1);
        drop(render_pass);
        let command_buffer = encoder.finish();
        queue.submit(Some(command_buffer));
    }
}

fn compile_shader_module(device: &wgpu::Device) -> wgpu::ShaderModule {
    use std::borrow::Cow;
    let code = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/shaders.wgsl"));
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(code)),
    })
}

fn create_display_pipeline(
    device: &wgpu::Device,
    shader_module: &wgpu::ShaderModule,
    surface_format: wgpu::TextureFormat,
) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // read-only binding
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // write-only storage binding
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba32Float,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
        ],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("display"),
        layout: Some(
            &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&bind_group_layout],
                ..Default::default()
            }),
        ),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Ccw,
            polygon_mode: wgpu::PolygonMode::Fill,
            ..Default::default()
        },
        vertex: wgpu::VertexState {
            module: shader_module,
            entry_point: "display_vs",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader_module,
            entry_point: "display_fs",
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format, // 使用传入的format而不是硬编码
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    (pipeline, bind_group_layout)
}

fn create_sample_texture(device: &wgpu::Device, width: u32, height: u32) -> [wgpu::Texture; 2] {
    let desc = wgpu::TextureDescriptor {
        label: Some("radiance samples"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    };
    [device.create_texture(&desc), device.create_texture(&desc)]
}

fn create_display_bind_groups(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    textures: &[wgpu::Texture; 2],
    uniform_buffer: &wgpu::Buffer,
) -> [wgpu::BindGroup; 2] {
    let views = [
        textures[0].create_view(&wgpu::TextureViewDescriptor::default()),
        textures[1].create_view(&wgpu::TextureViewDescriptor::default()),
    ];
    [
        // Bind group with view[0] assigned to binding 1 and view[1] assigned to binding 2.
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&views[0]),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&views[1]),
                },
            ],
        }),
        // Bind group with view[1] assigned to binding 1 and view[0] assigned to binding 2.
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&views[1]),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&views[0]),
                },
            ],
        }),
    ]
}
