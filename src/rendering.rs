extern crate imgui_winit_support;
use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig};

use std::time::{Instant, Duration};

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
}

pub struct Instance {
    position: cgmath::Vector3<f32>,
    //rotation: cgmath::Quaternion<f32>,
    frame: u32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
    frame: u32,
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position)).into(),
            frame: self.frame,
        }
    }
}

impl InstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We don't have to do this in code though.
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float4,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float4,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float4,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Uint,
                },
            ],
        }
    }
}

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
    sprite_rects: [[f32; 4]; 24],
}

impl Uniforms {
    fn new() -> Self {
        let mut sprite_rects: [[f32; 4]; 24] = [[0.0, 0.0, 0.0, 0.0]; 24];

        for i in 0..24 {
            sprite_rects[i] = [(i % 6) as f32 / 6.0, (i / 6) as f32 / 4.0, ((i % 6) + 1) as f32 / 6.0, ((i / 6) + 1) as f32 / 4.0];
        }

        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
            sprite_rects,
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
    attributes: &wgpu::vertex_attr_array![0 => Float3, 1 => Float3, ...],
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
            ]
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, -0.5, 0.0] },
    Vertex { position: [0.5, 0.5, 0.0] },
    Vertex { position: [-0.5, 0.5, 0.0] },
    Vertex { position: [0.5, -0.5, 0.0] },
];


const INDICES: &[u16] = &[
    0, 1, 2,
    0, 3, 1,
];

pub struct GameState {
    pub last_frame: Instant,
    pub time_delta: Option<Duration>,
    pub last_cursor: Option<(u32, u32)>,
    pub current_sprite_frame: u32,
    pub sprite_frame_count: u32,
    pub last_sprite_frame_time: Instant,
}

pub struct ImguiState {
    pub ctx: imgui::Context,
    pub renderer: Renderer,
    pub platform: imgui_winit_support::WinitPlatform,
    demo_open: bool,
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
    pub game: GameState,
    pub instances: Vec<Instance>,
    pub instance_buffer: wgpu::Buffer,
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
                power_preference: wgpu::PowerPreference::HighPerformance,
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
            present_mode: wgpu::PresentMode::Immediate,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        // 6x4 sprites in 600x400 pixels
        let diffuse_bytes = include_bytes!("trump_run.png");
        let diffuse_texture = texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "trump_run").unwrap();

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

            let imgui_renderer = Renderer::new(&mut imgui, &device, &queue, renderer_config);

            ImguiState{ctx: imgui, renderer: imgui_renderer, platform, demo_open: false}
        };

        let camera = camera::Camera {
            center: cgmath::Vector2::new(0.0, 0.0),
            height: 3.0,
            aspect: sc_desc.width as f32 / sc_desc.height as f32,
            znear: -1.0,
            zfar: 100.0,
        };

        let camera_controller = camera::CameraController::new(0.2);

        let mut uniforms = Uniforms::new();

        for elem in uniforms.sprite_rects.iter() {
            println!("Sprite coordinates");
            println!("{} {} {} {}", elem[0], elem[1], elem[2], elem[3]);
        }

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
                },
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

        let mut instances: Vec<Instance> = Vec::<Instance>::new();

        for i in 0..1 {
            instances.push(Instance{ position: cgmath::Vector3 {x: -1.0 + (i % 6) as f32, y: (i / 6) as f32, z: 0.0}, frame: i % 24 });
        }

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();

        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            }
        );

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
                vertex_buffers: &[Vertex::desc(), InstanceRaw::desc()],
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
            game: GameState{last_frame: Instant::now(), time_delta: None, last_cursor: None, current_sprite_frame: 0, sprite_frame_count: 24, last_sprite_frame_time: Instant::now()},
            instances,
            instance_buffer,
        }
    }

    // To support resizing, we need to re-create the swap chain on resize event
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        println!("Resizing");
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
        // Update time delta
        let new_frame = Instant::now();
        let dt = new_frame - self.game.last_frame;
        self.game.time_delta = Some(dt);
        self.game.last_frame = new_frame;

        if dt.as_millis() > 0 {
            self.camera_controller.update_camera(&mut self.camera);
            self.uniforms.update_view_proj(&self.camera);
            self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.uniforms]));

            if self.game.last_sprite_frame_time.elapsed().as_millis() > 100 {
                self.game.current_sprite_frame = (self.game.current_sprite_frame + 1) % self.game.sprite_frame_count;
                self.instances[0].frame = self.game.current_sprite_frame;
                let instance_data = self.instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
                self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instance_data));
                self.game.last_sprite_frame_time = Instant::now();
            }
        }
    }

    pub fn create_render_encoder(&mut self, frame: &wgpu::SwapChainTexture, winit_window: &Window) -> wgpu::CommandEncoder {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

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
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..));
            render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as _);

            self.imgui.platform.prepare_frame(self.imgui.ctx.io_mut(), &winit_window)
                .expect("Failed to prepare frame");

            let ui = self.imgui.ctx.frame();

            let window = imgui::Window::new(im_str!("Hello world!"));
            let mut tmp_color = self.bg_color;
            let time_delta_ms = match self.game.time_delta { Some(dur) => dur.as_millis(), None => 1 };

            window
                .always_auto_resize(true)
                .build(&ui, || {

                    let style = ui.push_style_vars([StyleVar::ItemSpacing([5.0, 5.0])].iter());

                    ui.text(im_str!("Frametime: {:?}ms, FPS: {:?}", time_delta_ms, 1000 / time_delta_ms));
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

            self.imgui.platform.prepare_render(&ui, &winit_window);

            self.imgui.renderer
                .render(ui.render(), &self.queue, &self.device, &mut render_pass)
                .expect("Rendering failed");
        }

        encoder
    }

    // We need Texture and TextureView to render the image
    pub fn render(&mut self, winit_window: &Window) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;

        let encoder = self.create_render_encoder(&frame, winit_window);

        // We can't call encoder.finish() until we release mutable borrow (drop)
        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
