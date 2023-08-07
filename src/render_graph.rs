pub mod attachment;
pub mod shader_builder;
pub mod pass_builder;
pub mod pipeline_builder;
pub mod handle_map;

use uuid::Uuid;
use petgraph::graph::{ NodeIndex, Graph };
use thiserror::Error;
use std::collections::HashMap;

use pass_builder::{ PassHandle, RenderPassBuilder };
use pipeline_builder::{ PipelineHandle, PipelineLayoutBuilder };
use attachment::{ AttachmentHandle, Attachment };
use handle_map::{ HandleType, HandleMap, Handle };

enum Vertex {
    Red(AttachmentHandle),
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

pub struct RenderGraph<'graph> {
    pipelines: HandleMap<PipelineHandle, PipelineLayoutBuilder<'graph>>,
    passes: HandleMap<PassHandle, RenderPassBuilder<'graph>>,
    attachments: HandleMap<AttachmentHandle, Attachment<'graph>>,
    graph: Graph<Vertex, ()>,
    vertex_handle_map: HashMap<Handle, VertexHandle>,
}

impl<'graph> RenderGraph<'graph> {
    pub fn new() -> RenderGraph<'graph> {
        RenderGraph {
            pipelines: HandleMap::new(),
            passes: HandleMap::new(),
            attachments: HandleMap::new(),
            graph: Graph::new(),
            vertex_handle_map: HashMap::new(),
        }
    }

    pub fn add_pipeline(&mut self, layout: PipelineLayoutBuilder<'graph>, id: Option<&str>) -> PipelineHandle {
        self.pipelines.add(layout, id.map(|id| id.to_string()))
    }

    pub fn add_render_pass(&mut self, pass: RenderPassBuilder<'graph>) -> (VertexHandle, Vec<VertexHandle>) {
        let pass_handle = self.passes.add(pass.clone(), pass.label.map(|l| l.to_string()));
        let pass_node = self.graph.add_node(Vertex::Blue(pass_handle));

        // Get all output attachments from this pass builder
        // First, create any new attachments we need
        let new_outputs: Vec<Attachment> = pass.colour_attachments.iter()
            .filter(|a| a.is_output())
            .filter(|a| a.is_new_resource())
            .map(|_| Attachment::Dynamic(Uuid::new_v4()))
            .inspect(|attachment| { self.attachments.add(*attachment, None); })
            .collect();

        // Get existing nodes from these attachments
        let existing_outputs: Vec<Attachment> = pass.colour_attachments.iter()
            .filter(|handle| handle.is_output())
            .filter_map(|handle| handle.resource_handle())
            .filter_map(|attachment_handle| self.attachments.get_from_handle(&attachment_handle))
            .map(|attachment| *attachment)
            .collect();

        // Attach this render pass to the outputs
        let mut outputs: Vec<VertexHandle> = existing_outputs.iter()
            .map(|attachment| self.add_resource(*attachment))
            .collect();
        outputs.append(
            &mut new_outputs.iter()
                .map(|attachment| self.add_resource(*attachment))
                .collect()
        );

        for vertex_handle in outputs.iter() {
            self.graph.add_edge(pass_node, vertex_handle.node_index, ());
        }
 
        // Attach inputs to this render pass
        pass.colour_attachments.iter()
            .filter_map(|handle| handle.resource_handle())
            .filter_map(|attachment_handle| self.vertex_handle_map.get(&attachment_handle))
            .for_each(|vertex_handle| { self.graph.add_edge(vertex_handle.node_index, pass_node, ()); });

        new_outputs.iter()
            .map(|attachment_handle| self.add_resource(attachment_handle.into_persistent()))
            .collect::<Vec<VertexHandle>>()
            .iter()
            .for_each(|vertex_handle| { self.graph.add_edge(vertex_handle.node_index, pass_node, ()); });

        let pass_vertex_handle = VertexHandle::new_from_node(pass_node, pass_handle);
        self.vertex_handle_map.insert(pass_handle, pass_vertex_handle);
        (pass_vertex_handle, outputs)
    }

    pub fn add_resource(&mut self, attachment: Attachment<'graph>) -> VertexHandle {
        let resource_handle = match attachment {
            Attachment::Persistent(id) => self.attachments.add(attachment, id.string_id.map(|s| s.to_string())),
            Attachment::Dynamic(_) => self.attachments.add(attachment, None)
        };

        let resource_node = self.graph.add_node(Vertex::Red(resource_handle));
        let resource_vertex_handle = VertexHandle::new_from_node(resource_node, resource_handle);
        self.vertex_handle_map.insert(resource_handle, resource_vertex_handle);
        resource_vertex_handle
    }

    pub fn string_graph(&self) -> Graph<String, String> {
        let get_resource_display = |handle| {
            let resource = self.attachments.get_from_handle(handle).unwrap();
            match resource {
                Attachment::Persistent(id) => id.string_id.map_or(id.global_id.to_string(), |s| s.to_string()),
                Attachment::Dynamic(uuid) => uuid.to_string()
            }
        };

        self.graph.map(|_, vertex| {
            let output = match vertex {
                Vertex::Red(resource_handle) => {
                    self.attachments.get_string_from_handle(resource_handle)
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

    pub fn compile(graph: &RenderGraph) {
        /* Algorithm:
         * 1. Find all Red sources. These are attachments that are external dependencies
         * 2. Find all Blue sources. These are passes with no defined input attachments:
         *  they may have output attachments that need to be created
         * 3. Assert that all external dependencies are satisfied
         * 4. Reverse directions and perform topological sort on graph
         * 5. From topological sort, if the texture is not an external dependency, create
         *  when needed
         */
    }
}
