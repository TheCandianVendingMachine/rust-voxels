pub mod attachment;
pub mod shader_builder;
pub mod pass_builder;
pub mod pipeline_builder;
pub mod handle_map;

use std::collections::HashMap;
use petgraph::graph::{ NodeIndex, Graph };
use thiserror::Error;

use pass_builder::{ PassHandle, RenderPassBuilder };
use pipeline_builder::{ PipelineHandle, PipelineLayoutBuilder };
use attachment::{ AttachmentHandle, Attachment };
use handle_map::{ HandleType, HandleMap };

enum Pass<'pass> {
    Render(RenderPassBuilder<'pass>)
}

enum Vertex<'vertex> {
    Red(Attachment),
    Blue(Pass<'vertex>)
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
    active_pass_map: HashMap<PassHandle, NodeIndex>,
    active_resource_map: HashMap<AttachmentHandle, NodeIndex>,
    graph: Graph<Vertex<'graph>, ()>
}

impl<'graph> RenderGraph<'graph> {
    pub fn new() -> RenderGraph<'graph> {
        RenderGraph {
            pipelines: HandleMap::new(),
            graph: Graph::new(),
            active_pass_map: HashMap::new(),
            active_resource_map: HashMap::new()
        }
    }

    pub fn add_pipeline(&mut self, layout: PipelineLayoutBuilder<'graph>, id: Option<&str>) -> PipelineHandle {
        self.pipelines.add(layout, id.map(|id| id.to_string()))
    }

    pub fn add_render_pass(&mut self, pass: RenderPassBuilder<'graph>) -> (PassHandle, Vec<AttachmentHandle>) {
        let new_resources: Vec<pass_builder::PassAttachment> = pass.colour_attachments.iter()
            .filter(|a| a.is_new_resource())
            .map(|a| *a)
            .collect();

        let mut resource_handles: Vec<AttachmentHandle> = pass.colour_attachments.iter()
            .filter_map(|a| a.resource_handle())
            .collect();

        for resource in new_resources {
            resource_handles.push(self.add_resource());
        }

        let handle = PassHandle::new();
        let node_index = self.graph.add_node(Vertex::Blue(Pass::Render(pass)));
        self.active_pass_map.insert(handle, node_index);
        (handle, resource_handles)
    }

    pub fn add_resource(&mut self) -> AttachmentHandle {
        todo!()
    }

    pub fn link_resource_to_pass(&mut self, pass: &PassHandle, resources: &[AttachmentHandle]) -> Result<(), RenderGraphResult> {
        if !self.active_pass_map.contains_key(pass) {
            return Err(RenderGraphResult::PassDoesNotExist)
        }

        let pass_vertex = self.active_pass_map.get(pass).unwrap();
        for resource in resources.iter() {
            if !self.active_resource_map.contains_key(resource) {
                return Err(RenderGraphResult::ResourceDoesNotExist)
            }

            self.graph.add_edge(*self.active_resource_map.get(resource).unwrap(), *pass_vertex, ());
        }

        Ok(())
    }

    pub fn link_pass_to_resource(&mut self, resource: &AttachmentHandle, passes: &[PassHandle]) -> Result<(), RenderGraphResult> {
        if !self.active_resource_map.contains_key(resource) {
            return Err(RenderGraphResult::ResourceDoesNotExist)
        }

        let resource_vertex = self.active_resource_map.get(resource).unwrap();
        for pass in passes.iter() {
            if !self.active_pass_map.contains_key(pass) {
                return Err(RenderGraphResult::PassDoesNotExist)
            }

            self.graph.add_edge(*self.active_pass_map.get(pass).unwrap(), *resource_vertex, ());
        }

        Ok(())
    }
}
