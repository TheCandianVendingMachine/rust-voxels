use crate::render_graph::resource::ResourceHandle;
use crate::render_graph::pipeline_builder::PipelineHandle;
pub use crate::render_graph::handle_map::Handle as PassHandle;

#[derive(Debug, Clone, Copy)]
pub enum PassResource {
    OnlyInput(ResourceHandle),
    OnlyOutput(Option<ResourceHandle>),
    InputAndOutput(ResourceHandle)
}

impl PassResource {
    pub fn is_output(&self) -> bool {
        match self {
            PassResource::OnlyOutput(_) => true,
            PassResource::InputAndOutput(_) => true,
            PassResource::OnlyInput(_) => false
        }
    }

    pub fn is_input(&self) -> bool {
        match self {
            PassResource::OnlyOutput(_) => false,
            PassResource::InputAndOutput(_) => true,
            PassResource::OnlyInput(_) => true
        }
    }

    pub fn is_new_resource(&self) -> bool {
        if let PassResource::OnlyOutput(resource) = *self {
            resource.is_none()
        } else {
            false
        }
    }

    pub fn resource_handle(&self) -> Option<ResourceHandle> {
        match *self {
            PassResource::OnlyOutput(resource) => resource,
            PassResource::OnlyInput(resource) => Some(resource),
            PassResource::InputAndOutput(resource) => Some(resource)
        }
    }
}

#[derive(Clone)]
pub struct RenderPassBuilder<'pass> {
    pub label: Option<&'pass str>,
    pub colour_attachments: Vec<PassResource>,
    pub depth_stencil: Option<PassResource>,
    pub vertex_buffer: Option<PassResource>,
    pub index_buffer: Option<PassResource>,
    pub pipeline: PipelineHandle,
}

impl<'pass> RenderPassBuilder<'pass> {
    pub fn render_pass(pipeline: PipelineHandle) -> Self {
        RenderPassBuilder {
            label: None,
            colour_attachments: Vec::new(),
            depth_stencil: None,
            vertex_buffer: None,
            index_buffer: None,
            pipeline
        }
    }

    pub fn label(mut self, label: &'pass str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn add_colour_attachment(mut self, attachment: PassResource) -> Self {
        self.colour_attachments.push(attachment);
        self
    }

    pub fn set_depth_stencil_attachment(mut self, depth_stencil: PassResource) -> Self {
        self.depth_stencil = Some(depth_stencil);
        self
    }

    pub fn set_vertex_buffer(mut self, vertex_buffer: PassResource) -> Self {
        self.vertex_buffer = Some(vertex_buffer);
        self
    }

    pub fn set_index_buffer(mut self, index_buffer: PassResource) -> Self {
        self.index_buffer = Some(index_buffer);
        self
    }
}
