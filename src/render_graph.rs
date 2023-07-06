pub mod shader_builder;
pub mod pass_builder;
pub mod pipeline_builder;

use std::collections::HashMap;
use petgraph::graph::{ NodeIndex, Graph };
use uuid::Uuid;
use thiserror::Error;

use pass_builder::PassBuilder;
use pipeline_builder::{ PipelineLayoutBuilder, BindGroupLayoutBuilder };

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

trait HandleType {
    fn new() -> Self;
    fn uuid(&self) -> Uuid;
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct ResourceHandle(Uuid);
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct PassHandle(Uuid);
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct PipelineLayoutHandle(Uuid);
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct BindGroupLayoutHandle(Uuid);

impl HandleType for PipelineLayoutHandle {
    fn new() -> Self {
        PipelineLayoutHandle(Uuid::new_v4())
    }

    fn uuid(&self) -> Uuid {
        self.0
    }
}

impl HandleType for BindGroupLayoutHandle {
    fn new() -> Self {
        BindGroupLayoutHandle(Uuid::new_v4())
    }

    fn uuid(&self) -> Uuid {
        self.0
    }
}

struct LayoutMap<T, HandleT> 
    where HandleT: HandleType + Copy + std::hash::Hash + PartialEq + Eq  {
    string_map: HashMap<String, HandleT>,
    handle_map: HashMap<HandleT, T>
}

impl<T, HandleT> LayoutMap<T, HandleT> where 
    HandleT: HandleType + Copy + std::hash::Hash + PartialEq + Eq {
    pub fn new() -> Self {
        LayoutMap {
            string_map: HashMap::new(),
            handle_map: HashMap::new()
        }
    }

    pub fn add(&mut self, object: T, string_id: Option<String>) -> HandleT {
        let handle = HandleT::new();
        self.handle_map.insert(handle, object);
        if let Some(id) = string_id {
            self.string_map.insert(id, handle);
        }
        handle
    }

    pub fn get_from_string(&self, string_id: &String) -> Option<&T> {
        if !self.string_map.contains_key(string_id) {
            return None
        }
        let handle = self.string_map.get(string_id).unwrap();
        self.get_from_handle(&handle)
    }

    pub fn get_from_handle(&self, handle: &HandleT) -> Option<&T> {
        self.handle_map.get(handle)
    }
}

pub struct RenderGraph<'graph> {
    pipeline_layouts: LayoutMap<PipelineLayoutBuilder<'graph>, PipelineLayoutHandle>,
    bind_group_layouts: LayoutMap<BindGroupLayoutBuilder<'graph>, BindGroupLayoutHandle>,
    graph: Graph<Vertex, ()>,
    pass_map: HashMap<PassHandle, NodeIndex>,
    resource_map: HashMap<ResourceHandle, NodeIndex>
}

impl RenderGraph<'_> {
    pub fn new<'a>() -> RenderGraph<'a> {
        RenderGraph {
            pipeline_layouts: LayoutMap::new(),
            bind_group_layouts: LayoutMap::new(),
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
