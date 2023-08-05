pub struct BindingGroupLayout<'binding> {
    pub label: Option<&'binding str>,
    pub entries: Vec<wgpu::BindGroupLayoutEntry>
}

impl BindingGroupLayout<'_> {
    pub fn create(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: self.label,
            entries: self.entries.as_slice()
        })
    }
}

pub struct PipelineLayout<'layout> {
    pub label: Option<&'layout str>,
    pub binding_group: BindingGroupLayout<'layout>
}

impl PipelineLayout<'_> {
    pub fn create(&self, device: &wgpu::Device) -> wgpu::PipelineLayout {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: self.label,
            push_constant_ranges: &[],
            bind_group_layouts: &[&self.binding_group.create(device)]
        })
    }
}
