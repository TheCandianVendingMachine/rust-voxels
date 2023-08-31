use std::borrow::Cow;
use std::collections::HashMap;
use crate::render_graph::resource::ResourceHandle;
pub use crate::render_graph::handle_map::Handle as ShaderHandle;

#[derive(Eq, PartialEq, Hash)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute
}

pub struct ShaderStageInputs {
    stage: ShaderStage,
    inputs: Vec<ResourceHandle>,
    representation: ShaderRepresentation
}

impl ShaderStageInputs {
    pub fn add_input(mut self, input: ResourceHandle) -> ShaderStageInputs {
        self.inputs.push(input);
        self
    }

    pub fn finish(mut self) -> ShaderRepresentation {
        self.representation.stages.insert(self.stage, self.inputs).unwrap();
        self.representation
    }
}

pub struct ShaderRepresentation {
    stages: HashMap<ShaderStage, Vec<ResourceHandle>>
}

impl ShaderRepresentation {
    pub fn shader() -> ShaderRepresentation {
        ShaderRepresentation {
            stages: HashMap::new()
        }
    }

    pub fn add_stage(self, stage: ShaderStage) -> ShaderStageInputs {
        ShaderStageInputs {
            stage,
            inputs: Vec::new(),
            representation: self
        }
    }
}

pub trait ShaderSource<'shader> {
    fn build(&self) -> wgpu::ShaderSource<'shader>;
}

#[derive(Debug, Clone)]
pub struct ShaderBuilder<'shader, S> where
    S: ShaderSource<'shader> + std::fmt::Debug + Clone {
    label: Option<&'shader str>,
    shader: &'shader S
}

impl<'shader, S> ShaderBuilder<'shader, S> where
    S: ShaderSource<'shader> + std::fmt::Debug + Clone { 
    pub fn shader(shader: &'shader S) -> Self {
        ShaderBuilder {
            label: None,
            shader
        }
    }

    pub fn label(mut self, label: &'shader str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn build(&self) -> wgpu::ShaderModuleDescriptor<'shader> {
        wgpu::ShaderModuleDescriptor {
            label: self.label,
            source: self.shader.build()
        }
    }
}

#[derive(Debug, Clone)]
pub struct WgslBuilder<'shader> {
    source: Cow<'shader, str>
}

impl<'shader> ShaderSource<'shader> for WgslBuilder<'shader> {
    fn build(&self) -> wgpu::ShaderSource<'shader> {
        wgpu::ShaderSource::Wgsl(self.source.clone())
    }
}

impl WgslBuilder<'_> {
    pub fn from_buffer(source: &'static str) -> WgslBuilder {
        WgslBuilder {
            source: Cow::from(source)
        }
    }
}
