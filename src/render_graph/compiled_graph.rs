use std::collections::HashMap;
use wgpu::{
    self,
    Device,
    PipelineLayout,
    RenderPass,
    RenderPipeline,
    ShaderModule,
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

pub struct CompiledGraph<'a> {
    shaders: HashMap<Uuid, ShaderModule>,
    pipeline_layouts: HashMap<Uuid, PipelineLayout>,
    render_pipelines: HashMap<Uuid, RenderPipeline>,
    render_passes: HashMap<Uuid, RenderPass<'a>>
}

impl<'graph> CompiledGraph<'graph> {
    const VERTEX_SHADER_ENTRY: &'static str = "vs_main";
    const FRAGMENT_SHADER_ENTRY: &'static str = "fs_main";
    const PRIMITIVE_STATE: wgpu::PrimitiveState = wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: Some(wgpu::Face::Back),
        polygon_mode: wgpu::PolygonMode::Fill,
        unclipped_depth: false,
        conservative: false
    };

    pub fn compile_from_definition<'compile_graph, 'device: 'compile_graph, S>(
        graph: &'compile_graph super::RenderGraph,
        device: &'device wgpu::Device,
        shaders: HashMap<ShaderHandle, &ShaderBuilder<'compile_graph, S>>,
        vertex_buffer_layout: &'compile_graph [wgpu::VertexBufferLayout],
        colour_target_state: &'compile_graph [Option<wgpu::ColorTargetState>]
    ) -> CompiledGraph<'compile_graph> where
        S: Clone + std::fmt::Debug + ShaderSource<'compile_graph> {
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
            render_passes: HashMap::new()
        };
        let nodes_to_visit = petgraph::algo::toposort(&graph.graph.reverse_graph, None).unwrap();

        let mut pipeline_layouts = HashMap::new();

        for node_index in nodes_to_visit {
            let v = graph.graph.forward_graph.node_weight(node_index).unwrap();
            match v {
                Vertex::Red(resource_handle) => {
                },
                Vertex::Blue(pass_handle) => {
                    let pass = graph.passes.get_from_handle(pass_handle).unwrap();
                    let pipeline = graph.pipelines.get_from_handle(&pass.pipeline).unwrap();
                    if !pipeline_layouts.contains_key(&pass.pipeline) {
                        pipeline_layouts.insert(pass.pipeline, pipeline.builder.clone().build());
                    }
                    let pipeline_layout = pipeline_layouts.get_mut(&pass.pipeline).unwrap();
                    // Create wgpu pipeline if it doesnt exist already
                    compiled_graph.create_pipeline(
                        pass,
                        pipeline,
                        pipeline_layout,
                        device,
                        &shaders,
                        vertex_buffer_layout,
                        colour_target_state
                    );

                    // Create render pass from pipeline
                    compiled_graph.create_render_pass();
                },
            }
        }

        compiled_graph
    }

    fn create_render_pass(
        &mut self
    ) {
    }

    fn create_pipeline<'pipeline, S>(
        &'pipeline mut self,
        pass_builder: &RenderPassBuilder,
        pipeline_info: &PipelineInfo,
        pipeline_layout: &mut render::PipelineLayout<'graph>,
        device: &'graph wgpu::Device,
        shaders: &HashMap<ShaderHandle, &ShaderBuilder<'graph, S>>,
        vertex_buffer_layout: &'graph [wgpu::VertexBufferLayout],
        colour_target_state: &'graph [Option<wgpu::ColorTargetState>]
    ) -> &'pipeline RenderPipeline where
        S: Clone + std::fmt::Debug + ShaderSource<'graph>,
    {
        if !self.render_pipelines.contains_key(&pass_builder.pipeline.uuid()) {
            return self.render_pipelines.get(&pass_builder.pipeline.uuid()).unwrap()
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

        self.add_render_pipeline(
            device,
            pass_builder.pipeline.uuid(),
            Some(ResourcePair::new(pass_builder.pipeline.uuid(), pipeline_layout)),
            vertex_shader,
            fragment_shader
        )
    }

    fn add_render_pipeline<'pipeline, VS, FS>(
        &'pipeline mut self,
        device: &wgpu::Device,
        render_pipeline_id: Uuid,
        mut render_pipeline_layout_builder: Option<ResourcePair<&mut render::PipelineLayout<'graph>>>,
        vertex_shader_builder: ShaderData<'graph, wgpu::VertexBufferLayout, VS>,
        fragment_shader_builder: Option<ShaderData<'graph, Option<wgpu::ColorTargetState>, FS>>,
    ) -> &'pipeline RenderPipeline where
        VS: ShaderSource<'graph> + std::fmt::Debug + Clone,
        FS: ShaderSource<'graph> + std::fmt::Debug + Clone,
    {
        if !self.render_pipelines.contains_key(&render_pipeline_id) {
            if !self.shaders.contains_key(&vertex_shader_builder.module_builder.id) {
                self.shaders.insert(
                    vertex_shader_builder.module_builder.id,
                    device.create_shader_module(vertex_shader_builder.module_builder.resource.build())
                );
            }

            if let Some(fs) = &fragment_shader_builder {
                if !self.shaders.contains_key(&fs.module_builder.id) {
                    self.shaders.insert(
                        fs.module_builder.id,
                        device.create_shader_module(fs.module_builder.resource.build())
                    );
                }
            }

            if let Some(layout) = &mut render_pipeline_layout_builder {
                if !self.pipeline_layouts.contains_key(&layout.id) {
                    self.pipeline_layouts.insert(
                        layout.id,
                        layout.resource.create(&device)
                    );
                }
            }

            let vertex_shader = self.shaders.get(&vertex_shader_builder.module_builder.id).unwrap();
            let fragment_shader = fragment_shader_builder.as_ref().map(
                |b| self.shaders.get(&b.module_builder.id).unwrap()
            );
            let pipeline_layout = render_pipeline_layout_builder.map(
                |b| self.pipeline_layouts.get(&b.id).unwrap()
            );

            let render_pipeline_descriptor = wgpu::RenderPipelineDescriptor {
                label: None,
                layout: pipeline_layout,
                vertex: wgpu::VertexState {
                    module: &vertex_shader,
                    entry_point: Self::VERTEX_SHADER_ENTRY,
                    buffers: vertex_shader_builder.inputs
                },
                fragment: fragment_shader.map(|fs|
                    wgpu::FragmentState {
                        module: &fs,
                        entry_point: Self::FRAGMENT_SHADER_ENTRY,
                        targets: fragment_shader_builder.unwrap().inputs,
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
                render_pipeline_id,
                device.create_render_pipeline(&render_pipeline_descriptor)
            );
        }
        self.render_pipelines.get(&render_pipeline_id).unwrap()
    }
}
