use std::collections::HashMap;
use wgpu::{
    PipelineLayout,
    RenderPass,
    RenderPipeline,
    ShaderModule,
    CommandEncoder,
    CommandBuffer
};
use uuid::Uuid;
use crate::render_graph::{
    shader_builder::{ ShaderBuilder, ShaderSource, ShaderHandle },
    pass_builder::RenderPassBuilder,
    resource::ResourceHandle,
    handle_map::HandleType,
    Vertex, PipelineInfo
};
use crate::render;

pub struct ResourcePair<T> {
    id: Uuid,
    resource: T
}

impl<T> ResourcePair<T> {
    pub fn new(id: Uuid, resource: T) -> ResourcePair<T> {
        ResourcePair {
            id, resource
        }
    }
}

pub struct ShaderData<'shader, I, S: Clone + std::fmt::Debug + ShaderSource<'shader>> {
    pub module_builder: ResourcePair<ShaderBuilder<'shader, S>>,
    pub inputs: &'shader [I]
}

pub struct CompiledGraph<'graph> {
    shaders: HashMap<Uuid, ShaderModule>,
    pipeline_layouts: HashMap<Uuid, PipelineLayout>,
    render_pipelines: HashMap<Uuid, RenderPipeline>,
    render_passes: HashMap<Uuid, RenderPass<'graph>>,
    render_queues: Vec<&'graph wgpu::Queue>,
}

impl<'graph> CompiledGraph<'graph> {
    const VERTEX_SHADER_ENTRY: &'static str = "vs_main";
    const FRAGMENT_SHADER_ENTRY: &'static str = "fs_main";
    const DEFAULT_CLEAR_COLOUR: wgpu::Color = wgpu::Color {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0
    };
    const PRIMITIVE_STATE: wgpu::PrimitiveState = wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: Some(wgpu::Face::Back),
        polygon_mode: wgpu::PolygonMode::Fill,
        unclipped_depth: false,
        conservative: false
    };

    pub fn render_from_graph<S>(
        graph: &'graph super::RenderGraph,
        device: &wgpu::Device,
        queues: &[&'graph render::Queue],
        shaders: &HashMap<ShaderHandle, &ShaderBuilder<'graph, S>>,
        vertex_buffer_layout: &'graph [wgpu::VertexBufferLayout],
        colour_target_state: &'graph [Option<wgpu::ColorTargetState>],
        vertex_buffer_attachments: &HashMap<ResourceHandle, wgpu::BufferSlice>,
        colour_attachments: &HashMap<ResourceHandle, wgpu::RenderPassColorAttachment>
    ) where
        S: Clone + std::fmt::Debug + ShaderSource<'graph> {
        /* Algorithm:
         * 1. Reverse directions and perform topological sort on graph
         * 2. From topological sort, if the resource is not an external dependency, create
         *  when needed. If the resource cannot be created (Input and a vertex buffer, for
         *  example), then panic
         */
        let mut compiled_graph = CompiledGraph {
            shaders: HashMap::new(),
            pipeline_layouts: HashMap::new(),
            render_pipelines: HashMap::new(),
            render_passes: HashMap::new(),
            render_queues: queues.iter().filter_map(
                |queue| {
                    if let render::Queue::Render(wgpu_queue) = queue {
                        return Some(wgpu_queue)
                    }
                    None
                }
            ).collect(),
        };

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compiled Graph Encoder")
        });
        let nodes_to_visit = petgraph::algo::toposort(&graph.graph.reverse_graph, None).unwrap();

        let mut pipeline_layouts = HashMap::new();

        for node_index in nodes_to_visit {
            let v = graph.graph.forward_graph.node_weight(node_index).unwrap();
            match v {
                Vertex::Red(resource_handle) => {
                    todo!();
                },
                Vertex::Blue(pass_handle) => {
                    let pass = graph.passes.get_from_handle(pass_handle).unwrap();
                    let pipeline_info = graph.pipelines.get_from_handle(&pass.pipeline).unwrap();
                    if !pipeline_layouts.contains_key(&pass.pipeline) {
                        pipeline_layouts.insert(pass.pipeline, pipeline_info.builder.clone().build());
                    }
                    let pipeline_layout = pipeline_layouts.get_mut(&pass.pipeline).unwrap();
                    // Create wgpu pipeline if it doesnt exist already
                    compiled_graph.create_pipeline(
                        pass,
                        pipeline_info,
                        pipeline_layout,
                        device,
                        &shaders,
                        vertex_buffer_layout,
                        colour_target_state
                    );

                    // Create render pass from pipeline
                    compiled_graph.create_render_pass(
                        device,
                        &mut encoder,
                        pass,
                        vertex_buffer_attachments,
                        colour_attachments
                    );
                },
            }
        }

        compiled_graph.render_queues[0].submit(std::iter::once(encoder.finish()));
    }

    fn create_render_pass<'render_pass>(
        &'render_pass mut self,
        device: &wgpu::Device,
        encoder: &mut CommandEncoder,
        render_pass: &RenderPassBuilder,
        vertex_buffer_attachments: &HashMap<ResourceHandle, wgpu::BufferSlice>,
        colour_attachments: &HashMap<ResourceHandle, wgpu::RenderPassColorAttachment>
    ) {
        let pipeline = self.render_pipelines.get(&render_pass.pipeline.uuid()).unwrap();
        let attachments: Vec<Option<wgpu::RenderPassColorAttachment>> = render_pass.colour_attachments.iter()
            .map(|h| Some(colour_attachments.get(&h.resource_handle().unwrap()).unwrap().clone()))
        .collect();

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &attachments,
            depth_stencil_attachment: None
        });

        render_pass.set_pipeline(&pipeline);
        render_pass.draw(0..3, 0..1);
    }

    fn create_pipeline<S>(
        &mut self,
        pass_builder: &RenderPassBuilder,
        pipeline_info: &PipelineInfo,
        pipeline_layout: &mut render::PipelineLayout<'graph>,
        device: &wgpu::Device,
        shaders: &HashMap<ShaderHandle, &ShaderBuilder<'graph, S>>,
        vertex_buffer_layout: &'graph [wgpu::VertexBufferLayout],
        colour_target_state: &'graph [Option<wgpu::ColorTargetState>]
    ) where
        S: Clone + std::fmt::Debug + ShaderSource<'graph>,
    {
        if !self.render_pipelines.contains_key(&pass_builder.pipeline.uuid()) {
            return
        }

        let vertex_shader = ShaderData {
            module_builder: ResourcePair::new(
                pipeline_info.vertex_shader.uuid(),
                (*shaders.get(&pipeline_info.vertex_shader).unwrap()).clone()
            ),
            inputs: vertex_buffer_layout
        };

        let fragment_shader = pipeline_info.fragment_shader.map(
            |fs| { 
                ShaderData {
                    module_builder: ResourcePair::new(
                        fs.uuid(),
                        (*shaders.get(&fs).unwrap()).clone()
                    ),
                    inputs: colour_target_state
                }
            }
        );

        if !self.shaders.contains_key(&vertex_shader.module_builder.id) {
            self.shaders.insert(
                vertex_shader.module_builder.id,
                device.create_shader_module(vertex_shader.module_builder.resource.build())
            );
        }

        if let Some(fs) = &fragment_shader {
            if !self.shaders.contains_key(&fs.module_builder.id) {
                self.shaders.insert(
                    fs.module_builder.id,
                    device.create_shader_module(fs.module_builder.resource.build())
                );
            }
        }

        if !self.pipeline_layouts.contains_key(&pass_builder.pipeline.uuid()) {
            self.pipeline_layouts.insert(
                pass_builder.pipeline.uuid(),
                pipeline_layout.create(&device)
            );
        }

        let vertex_shader_module = self.shaders.get(&vertex_shader.module_builder.id).unwrap();
        let fragment_shader_module = fragment_shader.as_ref().map(
            |b| self.shaders.get(&b.module_builder.id).unwrap()
        );
        let pipeline_layout = self.pipeline_layouts.get(&pass_builder.pipeline.uuid()).unwrap();

        let render_pipeline_descriptor = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader_module,
                entry_point: Self::VERTEX_SHADER_ENTRY,
                buffers: vertex_shader.inputs
            },
            fragment: fragment_shader_module.map(|fs|
                wgpu::FragmentState {
                    module: &fs,
                    entry_point: Self::FRAGMENT_SHADER_ENTRY,
                    targets: fragment_shader.unwrap().inputs,
                },
            ),
            primitive: Self::PRIMITIVE_STATE,
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false
            },
            multiview: None
        };

        self.render_pipelines.insert(
            pass_builder.pipeline.uuid(),
            device.create_render_pipeline(&render_pipeline_descriptor)
        );
    }
}
