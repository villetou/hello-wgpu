use winit::{
    event::*,
    window::Window,
};

use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}


/**
 * Could be created also with the following but needs <'static>
 * wgpu::VertexBufferDescriptor {
    stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
    step_mode: wgpu::InputStepMode::Vertex,
    attributes: &wgpu::vertex_attr_array![0 => Float3, 1 => Float3],
} */
impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                }
            ]
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
    Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
    Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
];
 
pub struct State {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub pointer: (f64, f64),
    pub render_pipeline: wgpu::RenderPipeline,
    pub render_pipeline2: wgpu::RenderPipeline,
    pub alternative_pipeline: bool,
    pub vertex_buffer: wgpu::Buffer,
    pub num_vertices: u32,
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    shader_validation: true,
                },
                None, // Trace path
            )
            .await
            .unwrap();
        
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsage::VERTEX,
            }
        );

        // Describes how images are displayed to Surface
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb, // The screen format that is most widely available, should use the screens native format but there's no way to query it yet
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo, // Immediate, Mailbox, Fifo (listen to VBlank or not?)
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);
        // PresentModes
        // Immediate
        // The presentation engine does **not** wait for a vertical blanking period and
        // the request is presented immediately. This is a low-latency presentation mode,
        // but visible tearing may be observed. Will fallback to `Fifo` if unavailable on the
        // selected  platform and backend. Not optimal for mobile.
        // Mailbox
        // The presentation engine waits for the next vertical blanking period to update
        // the current image, but frames may be submitted without delay. This is a low-latency
        // presentation mode and visible tearing will **not** be observed. Will fallback to `Fifo`
        // if unavailable on the selected platform and backend. Not optimal for mobile.

        // Fifo
        // The presentation engine waits for the next vertical blanking period to update
        // the current image. The framerate will be capped at the display refresh rate,
        // corresponding to the `VSync`. Tearing cannot be observed. Optimal for mobile.

        let vs_module = device.create_shader_module(wgpu::include_spirv!("../shader.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("../shader.frag.spv"));

        let vs_module2 = device.create_shader_module(wgpu::include_spirv!("../shader2.vert.spv"));
        let fs_module2 = device.create_shader_module(wgpu::include_spirv!("../shader2.frag.spv"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main", // 1.
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                // 2.
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }),
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[
                    Vertex::desc(),
                ],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let render_pipeline2 = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline 2"),
            layout: Some(&render_pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module2,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module2,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }),
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            pointer: (0.0, 0.0),
            render_pipeline,
            render_pipeline2,
            alternative_pipeline: false,
            vertex_buffer,
            num_vertices: VERTICES.len() as u32
        }
    }

    // To support resizing, we need to re-create the swap chain on resize event
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;

        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc)
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.pointer = (position.x, position.y);
                true
            }
            WindowEvent::KeyboardInput { input, .. } => match input {
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Space),
                    ..
                } => {
                    self.alternative_pipeline = !self.alternative_pipeline;
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    pub fn update(&mut self) {
        // todo!()
    }

    // We need Texture and TextureView to render the image
    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;

        // The command encoder will
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // encoder.begin_render_pass(...) borrows encoder mutably (aka &mut self).
        // We can't call encoder.finish() until we release that mutable borrow.
        // The {} around encoder.begin_render_pass(...) tells rust to drop any variables within them
        // thus releasing the mutable borrow on encoder and allowing us to finish() it.
        // You can also use drop(render_pass) to achieve the same effect.
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: &self.pointer.1 / f64::from(self.size.height),
                            g: 0.0,
                            b: &self.pointer.0 / f64::from(self.size.width),
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(
                match self.alternative_pipeline {
                    true => &self.render_pipeline,
                    false => &self.render_pipeline2
                }
            );

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.num_vertices, 0..1);
        }
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
