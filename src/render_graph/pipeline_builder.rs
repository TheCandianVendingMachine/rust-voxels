#[derive(Debug, Copy, Clone)]
struct BindGroupData {
    visibility: VisibilityBuilder,
    binding: wgpu::BindingType
}

#[derive(Debug, Copy, Clone)]
pub struct VisibilityBuilder {
    visibility_bits: u32
}

impl VisibilityBuilder {
    pub fn visibility() -> Self {
        VisibilityBuilder {
            visibility_bits: wgpu::ShaderStages::NONE.bits(),
        }
    }

    pub fn vertex(mut self) -> Self {
        self.visibility_bits |= wgpu::ShaderStages::VERTEX.bits();
        self
    }

    pub fn fragment(mut self) -> Self {
        self.visibility_bits |= wgpu::ShaderStages::FRAGMENT.bits();
        self
    }

    pub fn compute(mut self) -> Self {
        self.visibility_bits |= wgpu::ShaderStages::COMPUTE.bits();
        self
    }

    pub fn build(self) -> wgpu::ShaderStages {
        wgpu::ShaderStages::from_bits(self.visibility_bits).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct BindGroupBuilder<'binding> {
    label: Option<&'binding str>,
    bindings: Vec<BindGroupData>
}

impl<'binding> BindGroupBuilder<'binding> {
    pub fn binding() -> Self {
        BindGroupBuilder {
            label: None,
            bindings: Vec::new()
        }
    }

    pub fn add_binding(mut self, visibility: VisibilityBuilder, binding: wgpu::BindingType) -> Self {
        self.bindings.push(BindGroupData {
            visibility,
            binding
        });
        self
    }

    pub fn label(mut self, label: &'binding str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn build(self) -> wgpu::BindGroupLayoutDescriptor<'binding> {
        let entries: Vec<wgpu::BindGroupLayoutEntry> = self.bindings.iter()
            .enumerate()
            .map(|(index, binding)| wgpu::BindGroupLayoutEntry {
                binding: index as u32,
                visibility: binding.visibility.build(),
                ty: binding.binding,
                count: None,
            })
        .collect();

        wgpu::BindGroupLayoutDescriptor {
            label: self.label,
            entries: &[]
        }
    }
}

pub struct PipelineBuilder {

}

pub struct RenderPipelineBuilder {

}
