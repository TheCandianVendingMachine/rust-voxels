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
    pub binding_group: Option<BindingGroupLayout<'layout>>,
    pub bind_group_layouts_cache: Vec<wgpu::BindGroupLayout>,
}

impl PipelineLayout<'_> {
    pub fn create(&mut self, device: &wgpu::Device) -> wgpu::PipelineLayout {
        if let Some(binding_group) = &self.binding_group {
            self.bind_group_layouts_cache.push(binding_group.create(device))
        }

        let bind_group_refs: Vec<&wgpu::BindGroupLayout> = self.bind_group_layouts_cache.iter().map(|l| l).collect();
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: self.label,
            push_constant_ranges: &[],
            bind_group_layouts: bind_group_refs.as_slice()
        })
    }
}

#[derive(Copy, Clone)]
pub enum Attachment {

}

pub enum Queue {
    Compute(wgpu::Queue),
    Render(wgpu::Queue)
}
