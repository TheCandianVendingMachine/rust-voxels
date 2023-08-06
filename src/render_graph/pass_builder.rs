use crate::render_graph::attachment::AttachmentHandle;
use crate::render_graph::pipeline_builder::PipelineHandle;
pub use crate::render_graph::handle_map::Handle as PassHandle;

#[derive(Debug, Clone, Copy)]
pub enum PassAttachment {
    Input(AttachmentHandle),
    Output(Option<AttachmentHandle>),
    InputOutput(AttachmentHandle)
}

impl PassAttachment {
    pub fn is_output(&self) -> bool {
        if let PassAttachment::Output(_) = *self {
            true
        } else{
            false
        }
    }

    pub fn is_new_resource(&self) -> bool {
        if let PassAttachment::Output(resource) = *self {
            resource.is_some()
        } else {
            false
        }
    }

    pub fn resource_handle(&self) -> Option<AttachmentHandle> {
        match *self {
            PassAttachment::Output(resource) => resource,
            PassAttachment::Input(resource) => Some(resource),
            PassAttachment::InputOutput(resource) => Some(resource)
        }
    }
}

pub struct RenderPassBuilder<'pass> {
    label: Option<&'pass str>,
    pub colour_attachments: Vec<PassAttachment>,
    pub depth_stencil: Option<AttachmentHandle>,
    pub pipeline: PipelineHandle
}

impl<'pass> RenderPassBuilder<'pass> {
    pub fn render_pass(pipeline: PipelineHandle) -> Self {
        RenderPassBuilder {
            label: None,
            colour_attachments: Vec::new(),
            depth_stencil: None,
            pipeline
        }
    }

    pub fn label(mut self, label: &'pass str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn add_colour_attachment(mut self, attachment: PassAttachment) -> Self {
        self.colour_attachments.push(attachment);
        self
    }

    pub fn set_depth_stencil_attachment(mut self, depth_stencil: AttachmentHandle) -> Self {
        self.depth_stencil = Some(depth_stencil);
        self
    }
}
