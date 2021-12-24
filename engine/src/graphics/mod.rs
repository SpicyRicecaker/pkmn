pub mod buffers;
mod camera;
mod font;
pub mod render;
mod texture;

use camera::Camera;
use wgpu::{util::DeviceExt, BufferDescriptor};

use self::buffers::{Uniforms, Vertex};

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

pub mod camera_controller;
pub mod color;
pub mod image;

use std::f32::consts::PI;
use color::Color;

impl State {
    /// Takes in top left coordinate of square, width, and a `color::Color`
    pub fn draw_square(&mut self, x: f32, y: f32, width: f32, color: Color) {
        let color = wgpu::Color::from(color);
        let color = [
            color.r as f32,
            color.g as f32,
            color.b as f32,
            color.a as f32,
        ];
        // We're allowed to pass in coords straight from our game, since our view matrix
        // will take care of transforming coords

        // Z is always 0 for a 2d game
        let vertices = &[
            // Top left, 0
            Vertex {
                position: [x, y, 0.0],
                color,
            },
            // Top right, 1
            Vertex {
                position: [x + width, y, 0.0],
                color,
            },
            // Bot left, 2
            Vertex {
                position: [x, y + width, 0.0],
                color,
            },
            // bot right, 3
            Vertex {
                position: [x + width, y + width, 0.0],
                color,
            },
        ];

        let indices = &[
            0, 2, 3, // Top triangle
            3, 1, 0, // Bot triangle
        ];

        self.push_shape(vertices, indices);
    }

    pub fn draw_rectangle(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        let color = wgpu::Color::from(color);
        let color = [
            color.r as f32,
            color.g as f32,
            color.b as f32,
            color.a as f32,
        ];
        // We're allowed to pass in coords straight from our game, since our view matrix
        // will take care of transforming coords

        // Z is always 0 for a 2d game
        let vertices = &[
            // Top left, 0
            Vertex {
                position: [x, y, 0.0],
                color,
            },
            // Top right, 1
            Vertex {
                position: [x + width, y, 0.0],
                color,
            },
            // Bot left, 2
            Vertex {
                position: [x, y + height, 0.0],
                color,
            },
            // bot right, 3
            Vertex {
                position: [x + width, y + height, 0.0],
                color,
            },
        ];

        let indices = &[
            0, 2, 3, // Top triangle
            3, 1, 0, // Bot triangle
        ];

        self.push_shape(vertices, indices);
    }

    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
        let color = wgpu::Color::from(color);
        let color = [
            color.r as f32,
            color.g as f32,
            color.b as f32,
            color.a as f32,
        ];
        // Get angle of line
        let angle = ((y2 - y1) / (x2 - x1)).atan();
        // Get perpendicular upper angle of line
        let pangle = angle + PI / 2.0;
        let r = thickness / 2.0;
        // Get diffs
        let pdx = pangle.cos() * r;
        let pdy = pangle.sin() * r;

        let vertices = &[
            // Top left, 0
            Vertex {
                position: [x2 + pdx, y2 + pdy, 0.0],
                color,
            },
            // Top right, 1
            Vertex {
                position: [x1 + pdx, y1 + pdy, 0.0],
                color,
            },
            // bot right, 3
            Vertex {
                position: [x2 - pdx, y2 - pdy, 0.0],
                color,
            },
            // Bot left, 2
            Vertex {
                position: [x1 - pdx, y1 - pdy, 0.0],
                color,
            },
        ];


        let indices = &[
            0, 2, 3, // Top triangle
            3, 1, 0, // Bot triangle
        ];

        self.push_shape(vertices, indices);
    }

    /// Pushes a shape into the vector of shapes. These shapes are copied into the vertex and index buffer
    /// in the `render()` function, to be batch rendered.
    /// Internally, updates `num_indices` and `num_vertices`, as well as converts `indices` on shape based off of previous `num_indices`
    pub fn push_shape(&mut self, vertices: &[Vertex], indices: &[u16]) {
        let len = self.vertices.len() as u16;

        // Not sure which implementation is better/faster
        // indices.iter_mut().map(|i| *i += len);
        // self.state.indices.extend_from_slice(indices);
        // The reason is because while for_each avoids iterating over the
        // array twice, push() might increase/decrease array len
        // Need to benchmark

        indices.iter().for_each(|i| {
            self.indices.push(*i + len);
        });

        self.vertices.extend_from_slice(vertices);
    }

    pub fn clear_background(&mut self, color: color::Color) {
        self.background.clear(wgpu::Color::from(color));
    }
}