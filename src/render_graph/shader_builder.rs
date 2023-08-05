use std::borrow::Cow;

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
