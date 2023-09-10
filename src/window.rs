use winit::{
    event::*,
    event_loop::{ ControlFlow, EventLoop },
    window::{ self, WindowBuilder }
};

use crate::render;
use crate::render_graph::resource::Resource;
use crate::render_graph::shader_builder::{ ShaderHandle, ShaderStage, ShaderRepresentation, ShaderBuilder, WgslBuilder };
use crate::render_graph::pipeline_builder::PipelineLayoutBuilder;
use crate::render_graph::pass_builder::{ RenderPassBuilder, PassResource };
use crate::render_graph::RenderGraph;
use crate::render_graph::CompiledGraph;
use petgraph::dot::Dot;

use std::collections::HashMap;

struct State<'s> {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: render::Queue,
    config: wgpu::SurfaceConfiguration,
    shader_handle: ShaderHandle,
    shader: ShaderBuilder<'s, WgslBuilder<'s>>,
    render_graph: RenderGraph<'s>
}

impl State<'_> {
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

        let shader = ShaderBuilder::shader(WgslBuilder::from_file("triangle.wgsl"))
            .label("Shader");

        let mut render_graph = RenderGraph::new();
        let triangle_buffer = render_graph.add_resource(Resource::persistent_with_name("Triangle"));
        let depth_buffer = render_graph.add_resource(Resource::persistent_with_name("Depth"));
        let surface_handle = render_graph.add_resource(Resource::persistent_with_name("Surface"));
        let texture_input = render_graph.add_resource(Resource::persistent_with_name("Texture"));
        
        let shader_handle = render_graph.add_shader(
            ShaderRepresentation::shader()
                .add_stage(ShaderStage::Vertex).finish()
                .add_stage(ShaderStage::Fragment)
                    .add_input(surface_handle.handle)
                .finish(),
            Some("default_shader")
        );
        {
            let render_pipeline = render_graph.add_pipeline(
                PipelineLayoutBuilder::layout().label("Render Pipeline Layout"),
                shader_handle, Some(shader_handle),
                Some("render_pipeline")
            );

            let (main_pass, main_pass_outputs) = render_graph.add_render_pass(
                RenderPassBuilder::render_pass(render_pipeline)
                    .label("Triangle Pass")
                    //.add_colour_attachment(PassResource::OnlyInput(texture_input.handle))
                    .add_colour_attachment(PassResource::InputAndOutput(surface_handle.handle))
                    //.set_vertex_buffer(PassResource::OnlyInput(triangle_buffer.handle))
                    //.set_depth_stencil_attachment(PassResource::InputAndOutput(depth_buffer.handle))
            );

            let out_graph = render_graph.string_graph();
            let dot = Dot::new(&out_graph);
            std::fs::write("test.graph", format!("{:?}", dot)).unwrap();
        };

        State {
            surface,
            device,
            queue: render::Queue::Render(queue),
            config,
            shader_handle,
            shader,
            render_graph
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

        /*CompiledGraph::render_from_graph(
            &self.render_graph, &self.device,
            &[&self.queue],
            &HashMap::from([
                (self.shader_handle, self.shader)
            ]),
            &[],
            &[],
            &HashMap::new(),
            &HashMap::new()
        );*/
        output.present();

        Ok(())
    }
}

pub struct Window<'s> {
    state: State<'s>,
    size: winit::dpi::PhysicalSize<u32>,
    event_loop: Option<EventLoop<()>>,
    window: window::Window
}

impl Window<'_> {
    pub async fn new<'w>() -> Window<'w> {
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

