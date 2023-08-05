pub mod shader_builder;
pub mod pass_builder;
pub mod pipeline_builder;

use std::collections::HashMap;
use petgraph::graph::{ NodeIndex, Graph };
use uuid::Uuid;
use thiserror::Error;

use pass_builder::PassBuilder;

struct Resource {}
struct Pass {}

enum Vertex {
    Red(Resource),
    Blue(Pass)
}

#[derive(Debug, Error)]
pub enum RenderGraphResult {
    #[error("Resource was not created as a vertex")]
    ResourceDoesNotExist,
    #[error("Pass was not created as a vertex")]
    PassDoesNotExist
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct ResourceHandle(Uuid);
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct PassHandle(Uuid);

pub struct RenderGraph {
    graph: Graph<Vertex, ()>,
    pass_map: HashMap<PassHandle, NodeIndex>,
    resource_map: HashMap<ResourceHandle, NodeIndex>
}

impl RenderGraph {
    pub fn new() -> RenderGraph {
        RenderGraph {
            graph: Graph::new(),
            pass_map: HashMap::new(),
            resource_map: HashMap::new()
        }
    }

    pub fn add_pass(&mut self, pass: PassBuilder) -> (PassHandle, Vec<ResourceHandle>) {
        (PassHandle(Uuid::new_v4()), Vec::new())
    }

    pub fn add_resource(&mut self) -> ResourceHandle {
        ResourceHandle(Uuid::new_v4())
    }

    pub fn link_resource_to_pass(&mut self, pass: &PassHandle, resources: &[ResourceHandle]) -> Result<(), RenderGraphResult> {
        if !self.pass_map.contains_key(pass) {
            return Err(RenderGraphResult::PassDoesNotExist)
        }

        let pass_vertex = self.pass_map.get(pass).unwrap();
        for resource in resources.iter() {
            if !self.resource_map.contains_key(resource) {
                return Err(RenderGraphResult::ResourceDoesNotExist)
            }

            self.graph.add_edge(*self.resource_map.get(resource).unwrap(), *pass_vertex, ());
        }

        Ok(())
    }

    pub fn link_pass_to_resource(&mut self, resource: &ResourceHandle, passes: &[PassHandle]) -> Result<(), RenderGraphResult> {
        if !self.resource_map.contains_key(resource) {
            return Err(RenderGraphResult::ResourceDoesNotExist)
        }

        let resource_vertex = self.resource_map.get(resource).unwrap();
        for pass in passes.iter() {
            if !self.pass_map.contains_key(pass) {
                return Err(RenderGraphResult::PassDoesNotExist)
            }

            self.graph.add_edge(*self.pass_map.get(pass).unwrap(), *resource_vertex, ());
        }

        Ok(())
    }
}
