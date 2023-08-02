use std::borrow::Cow;

pub trait ShaderSource<'shader> {
    fn build(&self) -> wgpu::ShaderSource<'shader>;
}

pub struct ShaderBuilder<'shader> {
    label: Option<&'shader str>,
    shader: Option<&'shader dyn ShaderSource<'shader>>
}

impl<'shader> ShaderBuilder<'shader> { 
    pub fn new() -> ShaderBuilder<'shader> {
        ShaderBuilder {
            label: None,
            shader: None
        }
    }

    pub fn label(mut self, label: &'shader str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn shader(mut self, shader: &'shader impl ShaderSource<'shader>) -> Self {
        self.shader = Some(shader);
        self
    }

    pub fn build(self) -> wgpu::ShaderModuleDescriptor<'shader> {
        let shader = self.shader.unwrap();
        wgpu::ShaderModuleDescriptor {
            label: self.label,
            source: shader.build()
        }
    }
}

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
