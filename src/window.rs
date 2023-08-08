use winit::{
    event::*,
    event_loop::{ ControlFlow, EventLoop },
    window::{ self, WindowBuilder }
};

use crate::render_graph::resource::{ Resource, Id as ResourceId };
use crate::render_graph::shader_builder::{ ShaderBuilder, WgslBuilder };
use crate::render_graph::pipeline_builder::{ PipelineLayoutBuilder };
use crate::render_graph::pass_builder::{ RenderPassBuilder, PassResource };
use crate::render_graph::RenderGraph;
use petgraph::dot::Dot;
use uuid::Uuid;

fn create_render_graph<'a>() -> RenderGraph<'a> {
    let mut render_graph = RenderGraph::new();
    let render_pipeline = render_graph.add_pipeline(
        PipelineLayoutBuilder::layout().label("Render Pipeline Layout"),
        Some("render_pipeline")
    );

    let surface = render_graph.add_resource(Resource::Persistent(ResourceId::new_with_name("Surface")));
    let texture_input = render_graph.add_resource(Resource::Persistent(ResourceId::new_with_name("Texture")));
    let pp_input = render_graph.add_resource(Resource::Persistent(ResourceId::new_with_name("pp input")));
    let (main_pass, main_pass_outputs) = render_graph.add_render_pass(
        RenderPassBuilder::render_pass(render_pipeline)
            .label("Test Pass")
            .add_colour_attachment(PassResource::OnlyInput(texture_input.handle))
            .add_colour_attachment(PassResource::InputAndOutput(surface.handle))
    );
    let (cloud_pass, cloud_pass_outputs) = render_graph.add_render_pass(
        RenderPassBuilder::render_pass(render_pipeline)
            .label("Clouds")
            .add_colour_attachment(PassResource::OnlyInput(texture_input.handle))
            .add_colour_attachment(PassResource::OnlyOutput(None))
    );
    let (pp_pass, pp_pass_outputs) = render_graph.add_render_pass(
        RenderPassBuilder::render_pass(render_pipeline)
            .label("Post Processing")
            .add_colour_attachment(PassResource::OnlyInput(cloud_pass_outputs[0].handle))
            .add_colour_attachment(PassResource::OnlyInput(pp_input.handle))
            .add_colour_attachment(PassResource::InputAndOutput(main_pass_outputs[0].handle))
    );

    let out_graph = render_graph.string_graph();
    let dot = Dot::new(&out_graph);
    std::fs::write("test.graph", format!("{:?}", dot)).unwrap();

    render_graph
}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
}

impl State {
    async fn new(window: &window::Window) -> State {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default()
        });

        /* # Safety
         *
         * The  surface only needs to live as long as the window, and the window owns the
         * state so this will remain valid
         */
        let surface = unsafe { instance.create_surface(window) }.unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![]
        };
        surface.configure(&device, &config);

        let _render_graph = create_render_graph();

        let shader = device.create_shader_module(
            ShaderBuilder::shader(&WgslBuilder::from_buffer(include_str!("triangle.wgsl")))
            .label("Shader")
        .build());

        let mut render_pipeline_layout = PipelineLayoutBuilder::layout().label("Render Pipeline Layout").build();
        let render_pipeline_layout = render_pipeline_layout.create(&device);

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[]
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false
            },
            multiview: None
        });

        State {
            surface,
            device,
            queue,
            config,
            render_pipeline
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0
                        }),
                        store: true
                    }
                })],
                depth_stencil_attachment: None
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub struct Window {
    state: State,
    size: winit::dpi::PhysicalSize<u32>,
    event_loop: Option<EventLoop<()>>,
    window: window::Window
}

impl Window {
    pub async fn new() -> Window {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();
        let size = window.inner_size();

        Window {
            state: State::new(&window).await,
            size,
            event_loop: Some(event_loop),
            window
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.state.render()
    }

    fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::Resized(physical_size) => {
                self.resize(*physical_size)
            },
            WindowEvent::ScaleFactorChanged{ new_inner_size, .. } => {
                self.resize(**new_inner_size)
            },
            _ => ()
        }
    }

    pub fn run(mut self) {
        let event_loop = self.event_loop.take().unwrap();
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id
            } if window_id == self.window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => self.handle_window_event(event)
            },
            Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                match self.state.render() {
                    Ok(_) => {},
                    Err(wgpu::SurfaceError::Lost) => self.state.resize(self.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e)
                }
            }
            Event::MainEventsCleared => self.window.request_redraw(),
            _ => ()
        });
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.state.resize(self.size);
        }
    }
}

