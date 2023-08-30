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
    shader_builder::{ ShaderBuilder, ShaderSource },
    pass_builder::RenderPassBuilder,
    pipeline_builder::PipelineLayoutBuilder,
};

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

pub struct CompiledGraph<'a> {
    device: &'a Device,
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

    pub fn new(device: &'graph Device) -> CompiledGraph<'graph> {
        CompiledGraph {
            device,
            pipeline_layouts: HashMap::new(),
            shaders: HashMap::new(),
            render_pipelines: HashMap::new(),
            render_passes: HashMap::new()
        }
    }

    pub fn add_render_pipeline<VS, FS>(
        &'graph mut self,
        render_pipeline_id: Uuid,
        render_pipeline_layout_builder: Option<ResourcePair<PipelineLayoutBuilder>>,
        vertex_shader_builder: ResourcePair<ShaderBuilder<'graph, VS>>,
        fragment_shader_builder: Option<ResourcePair<ShaderBuilder<'graph, FS>>>,
    ) where
        VS: ShaderSource<'graph> + std::fmt::Debug + Clone,
        FS: ShaderSource<'graph> + std::fmt::Debug + Clone
    {
        if !self.render_pipelines.contains_key(&render_pipeline_id) {
            if !self.shaders.contains_key(&vertex_shader_builder.id) {
                self.shaders.insert(
                    vertex_shader_builder.id,
                    self.device.create_shader_module(vertex_shader_builder.resource.build())
                );
            }

            if let Some(fs) = &fragment_shader_builder {
                if !self.shaders.contains_key(&fs.id) {
                    self.shaders.insert(
                        fs.id,
                        self.device.create_shader_module(fs.resource.build())
                    );
                }
            }

            if let Some(layout) = &render_pipeline_layout_builder {
                if !self.pipeline_layouts.contains_key(&layout.id) {
                }
            }

            let vertex_shader = self.shaders.get(&vertex_shader_builder.id).unwrap();
            let fragment_shader = fragment_shader_builder.map(
                |b| self.shaders.get(&b.id).unwrap()
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
                    buffers: &[]
                },
                fragment: fragment_shader.map(|fs|
                    wgpu::FragmentState {
                        module: &fs,
                        entry_point: Self::FRAGMENT_SHADER_ENTRY,
                        targets: &[/*Some(wgpu::ColorTargetState {
                            format: config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })*/],
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
                self.device.create_render_pipeline(&render_pipeline_descriptor)
            );
        }
    }
}
