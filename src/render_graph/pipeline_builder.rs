use crate::render;

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
pub struct BindGroupLayoutBuilder<'binding> {
    label: Option<&'binding str>,
    bindings: Vec<BindGroupData>
}

impl<'binding> BindGroupLayoutBuilder<'binding> {
    pub fn binding() -> Self {
        BindGroupLayoutBuilder {
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

    pub fn build(self) -> render::BindingGroupLayout<'binding> {
        let entries: Vec<wgpu::BindGroupLayoutEntry> = self.bindings.iter()
            .enumerate()
            .map(|(index, binding)| wgpu::BindGroupLayoutEntry {
                binding: index as u32,
                visibility: binding.visibility.build(),
                ty: binding.binding,
                count: None,
            })
        .collect();

        render::BindingGroupLayout {
            label: self.label,
            entries
        }
    }
}

#[derive(Debug, Clone)]
pub struct PipelineLayoutBuilder<'layout> {
    label: Option<&'layout str>,
    bind_group: BindGroupLayoutBuilder<'layout>
}

impl<'layout> PipelineLayoutBuilder<'layout> {
    pub fn label(mut self, label: &'layout str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn bind_group(mut self, bind_group: BindGroupLayoutBuilder<'layout>) -> Self {
        PipelineLayoutBuilder {
            bind_group,
            label: None
        }
    }

    pub fn build(self) -> render::PipelineLayout<'layout> {
        render::PipelineLayout {
            label: self.label,
            binding_group: self.bind_group.build()
        }
    }
}

enum PipelineBuilderType {
    Render(RenderPipelineBuilder)
}

pub struct PipelineBuilder<'pipeline> {
    label: Option<&'pipeline str>,
    sub_builder: PipelineBuilderType
}

pub struct RenderPipelineBuilder {

}
