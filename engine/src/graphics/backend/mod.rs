pub mod buffers;
mod camera;
mod font;
pub mod render;
mod texture;

use camera::Camera;
use wgpu::{util::DeviceExt, BufferDescriptor};

use self::buffers::Uniforms;

pub struct State {
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    queue: wgpu::Queue,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,

    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,

    pub vertices: Vec<buffers::Vertex>,
    pub indices: Vec<u16>,

    pub camera: Camera,

    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,

    pub background: Background,

    pub font_interface: font::FontInterface,
}

impl State {
    pub async fn new(window: &winit::window::Window) -> Self {
        let size = window.inner_size();

        // First create the wgpu instance, choosing the primary backend
        // Currently only dx12 for compile times
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);

        // Create the surface to draw on (from window, which we get from winit)
        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Unable to find adapter");

        let (device, queue) = adapter
            // Create the device from adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .expect("Unable to create device");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            // Low latency vsync is mailbox, falls back to Fifo,
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &config);

        let camera = Camera::new(config.width as f32, config.height as f32);

        let mut uniforms = Uniforms::new(config.width as f32, config.height as f32);
        uniforms.update_view_proj(&camera);

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // recall, bindgroup are resources that the gpu can access through specified shaders
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    // layout
                    binding: 0,
                    // visible only to vertex stage shaders
                    visibility: wgpu::ShaderStages::VERTEX,
                    // ty = type of binding
                    ty: wgpu::BindingType::Buffer {
                        // uniform value buffer
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Uniform Bind Group Layout"),
            });

        // create uniform bind group
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("Uniform Bind Group"),
        });

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                // Specify the entry point function for shaders, set by [[stage(fragment)]]
                entry_point: "vs_main",
                // We should pass in info into the shader itself, right now we're creating it in the shader for hello world
                buffers: &[buffers::Vertex::desc()],
            },
            // Fragment technically opt
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                // Target color output for swap chain, replace old pixels, and write to all colors
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let vertices = Vec::new();
        let indices = Vec::new();

        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: 0,
            usage: wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Index Buffer"),
            size: 0,
            usage: wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let background = Background::default();

        let font_interface = font::FontInterface::new(&device, config.format);
        Self {
            surface,
            config,
            device,
            queue,
            size,
            camera,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            render_pipeline,
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
            background,
            font_interface,
        }
    }
}

pub struct Background {
    pub color: wgpu::Color,
    pub should_clear: bool,
}

impl Background {
    pub fn clear(&mut self, color: wgpu::Color) {
        self.should_clear = true;
        self.color = color;
    }
    pub fn reset(&mut self) {
        self.should_clear = false;
    }
}

impl Default for Background {
    fn default() -> Self {
        Background {
            color: wgpu::Color::TRANSPARENT,
            should_clear: false,
        }
    }
}
