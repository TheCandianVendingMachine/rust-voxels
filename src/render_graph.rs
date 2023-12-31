pub mod compiled_graph;
pub mod resource;
pub mod shader_builder;
pub mod pass_builder;
pub mod pipeline_builder;
pub mod handle_map;

pub use compiled_graph::CompiledGraph;

use uuid::Uuid;
use petgraph::graph::{ NodeIndex, Graph };
use thiserror::Error;
use std::collections::HashMap;

use pass_builder::{ PassHandle, RenderPassBuilder };
use pipeline_builder::{ PipelineHandle, PipelineLayoutBuilder };
use resource::{ ResourceHandle, Resource };
use shader_builder::{ ShaderHandle, ShaderRepresentation };
use handle_map::{ HandleType, HandleMap, Handle };

#[derive(Clone)]
enum Vertex {
    Red(ResourceHandle),
    Blue(PassHandle)
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct VertexHandle {
    node_index: NodeIndex,
    pub handle: Handle
}

impl VertexHandle {
    fn new_from_node(node_index: NodeIndex, handle: Handle) -> VertexHandle {
        VertexHandle {
            node_index,
            handle
        }
    }
}

#[derive(Debug, Error)]
pub enum RenderGraphResult {
    #[error("Resource was not created as a vertex")]
    ResourceDoesNotExist,
    #[error("Pass was not created as a vertex")]
    PassDoesNotExist
}

struct RenderGraphMeta {
    forward_graph: Graph<Vertex, ()>,
    reverse_graph: Graph<Vertex, ()>
}

impl RenderGraphMeta {
    fn new() -> RenderGraphMeta {
        RenderGraphMeta {
            forward_graph: Graph::new(),
            reverse_graph: Graph::new(),
        }
    }

    fn add_node(&mut self, v: Vertex) -> NodeIndex {
        self.forward_graph.add_node(v.clone());
        self.reverse_graph.add_node(v)
    }

    fn add_edge(&mut self, from: NodeIndex, to: NodeIndex) {
        self.forward_graph.add_edge(from, to, ());
        self.reverse_graph.add_edge(to, from, ());
    }
}

struct PipelineInfo<'info> {
    builder: PipelineLayoutBuilder<'info>,
    vertex_shader: ResourceHandle,
    fragment_shader: Option<ResourceHandle>
}

pub struct RenderGraph<'graph> {
    shaders: HandleMap<ShaderHandle, ShaderRepresentation>,
    pipelines: HandleMap<PipelineHandle, PipelineInfo<'graph>>,
    passes: HandleMap<PassHandle, RenderPassBuilder<'graph>>,
    resources: HandleMap<ResourceHandle, Resource<'graph>>,
    graph: RenderGraphMeta,
    vertex_handle_map: HashMap<Handle, VertexHandle>,
}

impl<'graph> RenderGraph<'graph> {
    pub fn new() -> RenderGraph<'graph> {
        RenderGraph {
            shaders: HandleMap::new(),
            pipelines: HandleMap::new(),
            passes: HandleMap::new(),
            resources: HandleMap::new(),
            graph: RenderGraphMeta::new(),
            vertex_handle_map: HashMap::new(),
        }
    }

    pub fn add_shader(&mut self, shader: ShaderRepresentation, id: Option<&str>) -> ShaderHandle {
        self.shaders.add(shader, id.map(|id| id.to_string()))
    }

    pub fn add_pipeline(&mut self,
                        layout: PipelineLayoutBuilder<'graph>,
                        vertex_shader: ResourceHandle,
                        fragment_shader: Option<ResourceHandle>,
                        id: Option<&str>
    ) -> PipelineHandle {
        self.pipelines.add(PipelineInfo {
                builder: layout,
                vertex_shader,
                fragment_shader
            }, id.map(|id| id.to_string())
        )
    }

    pub fn add_render_pass(&mut self, pass: RenderPassBuilder<'graph>) -> (VertexHandle, Vec<VertexHandle>) {
        let pass_handle = self.passes.add(pass.clone(), pass.label.map(|l| l.to_string()));
        let pass_node = self.graph.add_node(Vertex::Blue(pass_handle));

        let resource_iter = pass.colour_attachments.iter()
            .chain(pass.depth_stencil.iter())
            .chain(pass.vertex_buffer.iter())
            .chain(pass.index_buffer.iter());

        // Get all output resources from this pass builder
        // First, create any new resources we need
        let new_outputs: Vec<Resource> = resource_iter.clone()
            .filter(|a| a.is_output())
            .filter(|a| a.is_new_resource())
            .map(|_| Resource::Dynamic(Uuid::new_v4()))
            .inspect(|resource| { self.resources.add(*resource, None); })
            .collect();

        // Get existing nodes from these resources
        let existing_outputs: Vec<Resource> = resource_iter.clone()
            .filter(|handle| handle.is_output())
            .filter_map(|handle| handle.resource_handle())
            .filter_map(|resource_handle| self.resources.get_from_handle(&resource_handle))
            .map(|resource| *resource)
            .collect();

        // Attach this render pass to the outputs
        let mut outputs: Vec<VertexHandle> = existing_outputs.iter()
            .map(|resource| self.add_resource(*resource))
            .collect();
        outputs.append(
            &mut new_outputs.iter()
                .map(|resource| self.add_resource(*resource))
                .collect()
        );

        for vertex_handle in outputs.iter() {
            self.graph.add_edge(pass_node, vertex_handle.node_index);
        }
 
        // Attach inputs to this render pass
        resource_iter
            .filter_map(|handle| handle.resource_handle())
            .filter_map(|resource_handle| self.vertex_handle_map.get(&resource_handle))
            .for_each(|vertex_handle| { self.graph.add_edge(vertex_handle.node_index, pass_node); });

        new_outputs.iter()
            .map(|resource_handle| self.add_resource(resource_handle.into_persistent()))
            .collect::<Vec<VertexHandle>>()
            .iter()
            .for_each(|vertex_handle| { self.graph.add_edge(vertex_handle.node_index, pass_node); });

        let pass_vertex_handle = VertexHandle::new_from_node(pass_node, pass_handle);
        self.vertex_handle_map.insert(pass_handle, pass_vertex_handle);
        (pass_vertex_handle, outputs)
    }

    pub fn add_resource(&mut self, resource: Resource<'graph>) -> VertexHandle {
        let resource_handle = match resource {
            Resource::Persistent(id) => self.resources.add(resource, id.string_id.map(|s| s.to_string())),
            Resource::Dynamic(_) => self.resources.add(resource, None)
        };

        let resource_node = self.graph.add_node(Vertex::Red(resource_handle));
        let resource_vertex_handle = VertexHandle::new_from_node(resource_node, resource_handle);
        self.vertex_handle_map.insert(resource_handle, resource_vertex_handle);
        resource_vertex_handle
    }

    pub fn string_graph(&self) -> Graph<String, String> {
        let get_resource_display = |handle| {
            let resource = self.resources.get_from_handle(handle).unwrap();
            match resource {
                Resource::Persistent(id) => id.string_id.map_or(id.global_id.to_string(), |s| s.to_string()),
                Resource::Dynamic(uuid) => uuid.to_string()
            }
        };

        self.graph.forward_graph.map(|_, vertex| {
            let output = match vertex {
                Vertex::Red(resource_handle) => {
                    self.resources.get_string_from_handle(resource_handle)
                        .or(Some(get_resource_display(resource_handle)))
                    .unwrap()
                }
                Vertex::Blue(pass_handle) =>
                    self.passes.get_string_from_handle(pass_handle)
                        .or(Some(pass_handle.uuid().to_string()))
                    .unwrap()

            };
            output
        }, |_, _| "".to_string())
    }
}

