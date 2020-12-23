extern crate imgui_winit_support;
use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig};

use std::time::Instant;

mod texture;
mod camera;

use winit::{
    event::*,
    window::Window,
};

use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl Uniforms {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &camera::Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
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
                    format: wgpu::VertexFormat::Float2,
                }
            ]
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614], },
    Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354], },
    Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397057], },
    Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732911], },
    Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641], },
];


const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

pub struct ImguiState {
    pub ctx: imgui::Context,
    pub renderer: Renderer,
    pub platform: imgui_winit_support::WinitPlatform,
    demo_open: bool,
    last_frame: Instant,
    last_cursor: Option<(u32, u32)>,
}

pub struct State {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub pointer: (f64, f64),
    pub render_pipeline: wgpu::RenderPipeline,
    pub draw_challenge: bool,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer, 
    pub num_indices: u32,
    pub diffuse_texture: texture::Texture,
    pub diffuse_bind_group: wgpu::BindGroup,
    camera: camera::Camera,
    camera_controller: camera::CameraController,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    pub imgui: ImguiState,
    pub bg_color: [f32; 3],
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

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsage::INDEX,
            }
        );

        let num_indices = INDICES.len() as u32;

        // Describes how images are displayed to Surface
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb, // The screen format that is most widely available, should use the screens native format but there's no way to query it yet
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate, // Immediate, Mailbox, Fifo (listen to VBlank or not?)
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

        let diffuse_bytes = include_bytes!("happy-tree.png");
        let diffuse_texture = texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();

        let texture_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Uint,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                        },
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            }
        );

        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );

        // Set up dear imgui
        let imgui = {
            let mut imgui = imgui::Context::create();

            let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
            platform.attach_window(
                imgui.io_mut(),
                &window,
                imgui_winit_support::HiDpiMode::Default,
            );
            imgui.set_ini_filename(None);

            imgui.io_mut().mouse_pos = [0.0, 0.0];

            imgui.style_mut().window_border_size = 0.0;
            imgui.style_mut().window_padding = [10.0, 10.0];

            let hidpi_factor = window.scale_factor();

            let font_size = (16.0 * hidpi_factor) as f32;
            imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

            imgui.fonts().add_font(&[FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    oversample_h: 1,
                    pixel_snap_h: true,
                    size_pixels: font_size,
                    ..Default::default()
                }),
            }]);

            let renderer_config = RendererConfig {
                texture_format: sc_desc.format,
                ..Default::default()
            };

            let mut imgui_renderer = Renderer::new(&mut imgui, &device, &queue, renderer_config);

            let mut last_frame = Instant::now();
            let mut last_cursor = None;

            ImguiState{ctx: imgui, renderer: imgui_renderer, platform, demo_open: false, last_frame, last_cursor}
        };

        let camera = camera::Camera {
            eye: (0.0, 1.0, 2.0).into(), // +z is out of the screen
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: sc_desc.width as f32 / sc_desc.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let camera_controller = camera::CameraController::new(0.2);

        let mut uniforms = Uniforms::new();
        uniforms.update_view_proj(&camera);

        let uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            }
        );

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("uniform_bind_group_layout"),
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(..))
                }
            ],
            label: Some("uniform_bind_group"),
        });


        let vs_module = device.create_shader_module(wgpu::include_spirv!("shader.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("shader.frag.spv"));

        let render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &uniform_bind_group_layout,
                ],
                push_constant_ranges: &[],
            }
        );

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
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

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            pointer: (0.0, 0.0),
            render_pipeline,
            draw_challenge: false,
            vertex_buffer,
            index_buffer,
            num_indices,
            diffuse_bind_group,
            diffuse_texture,
            camera,
            camera_controller,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            imgui,
            bg_color: [0.02, 0.02, 0.01],
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
        self.camera_controller.process_events(event);

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
                    self.draw_challenge = !self.draw_challenge;
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.uniforms.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.uniforms]));
    }

    // We need Texture and TextureView to render the image
    pub fn render(&mut self, window: &Window) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;

        // Imgui stuff
        self.imgui.platform.prepare_frame(self.imgui.ctx.io_mut(), &window)
                    .expect("Failed to prepare frame");

        let ui = self.imgui.ctx.frame();

        {
            let window = imgui::Window::new(im_str!("Hello world!"));
            let mut tmp_color = self.bg_color;
            window
                .always_auto_resize(true)
                .build(&ui, || {

                    let style = ui.push_style_vars([StyleVar::ItemSpacing([4.0, 4.0])].iter());

                    ui.text(im_str!("Frametime: {:?}", 0.1337)); // delta_s
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(im_str!(
                        "Mouse Position: ({:.0},{:.0})",
                        mouse_pos[0],
                        mouse_pos[1]
                    ));
                    ui.separator();
                    if ColorEdit::new(im_str!("color_edit"), &mut tmp_color).build(&ui) {
                        // state.notify_text = "*** Red button was clicked";
                    }

                    style.pop(&ui);
                });

            self.bg_color = tmp_color;

            if self.imgui.demo_open == true {
                ui.show_demo_window(&mut self.imgui.demo_open);
            }
        }

        // The command encoder will
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.imgui.platform.prepare_render(&ui, &window);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: f64::from(self.bg_color[0]),
                            g: f64::from(self.bg_color[1]),
                            b: f64::from(self.bg_color[2]),
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..));
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);        
            
            self.imgui.renderer
                .render(ui.render(), &self.queue, &self.device, &mut render_pass)
                .expect("Rendering failed");

        }

        // We can't call encoder.finish() until we release mutable borrow (drop)
        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
