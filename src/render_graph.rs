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

struct HandleMap<T, HandleT> 
    where HandleT: HandleType + Copy + std::hash::Hash + PartialEq + Eq  {
    string_map: HashMap<String, HandleT>,
    handle_map: HashMap<HandleT, T>
}

impl<T, HandleT> HandleMap<T, HandleT> where 
    HandleT: HandleType + Copy + std::hash::Hash + PartialEq + Eq {
    pub fn new() -> Self {
        HandleMap {
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
    pipeline_layouts: HandleMap<PipelineLayoutBuilder<'graph>, PipelineLayoutHandle>,
    bind_group_layouts: HandleMap<BindGroupLayoutBuilder<'graph>, BindGroupLayoutHandle>,
    graph: Graph<Vertex, ()>,
    active_pass_map: HashMap<PassHandle, NodeIndex>,
    active_resource_map: HashMap<ResourceHandle, NodeIndex>
}

impl<'graph> RenderGraph<'graph> {
    pub fn new() -> RenderGraph<'graph> {
        RenderGraph {
            pipeline_layouts: HandleMap::new(),
            bind_group_layouts: HandleMap::new(),
            graph: Graph::new(),
            active_pass_map: HashMap::new(),
            active_resource_map: HashMap::new()
        }
    }

    pub fn add_pipeline_layout(&mut self, layout: PipelineLayoutBuilder<'graph>, id: Option<String>) -> PipelineLayoutHandle {
        self.pipeline_layouts.add(layout, id)
    }

    pub fn add_bind_group_layout(&mut self, layout: BindGroupLayoutHandle<'graph>, id: Option<String>) -> BindGroupLayoutHandle {
        self.bind_group_layouts.add(layout, id)
    }

    pub fn add_pass(&mut self, pass: PassBuilder) -> (PassHandle, Vec<ResourceHandle>) {
        (PassHandle(Uuid::new_v4()), Vec::new())
    }

    pub fn add_resource(&mut self) -> ResourceHandle {
        ResourceHandle(Uuid::new_v4())
    }

    pub fn link_resource_to_pass(&mut self, pass: &PassHandle, resources: &[ResourceHandle]) -> Result<(), RenderGraphResult> {
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

    pub fn link_pass_to_resource(&mut self, resource: &ResourceHandle, passes: &[PassHandle]) -> Result<(), RenderGraphResult> {
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
