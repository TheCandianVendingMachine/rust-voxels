use crate::render_graph::ResourceHandle;

#[derive(Clone, Copy, PartialEq)]
pub enum AttachmentType {
    Input(ResourceHandle),
    Output(Option<ResourceHandle>)
}

impl AttachmentType {
    pub fn is_output(&self) -> bool {
        if let AttachmentType::Output(_) = *self {
            true
        } else{
            false
        }
    }

    pub fn is_new_resource(&self) -> bool {
        if let AttachmentType::Output(resource) = *self {
            resource.is_some()
        } else {
            false
        }
    }

    pub fn resource_handle(&self) -> Option<ResourceHandle> {
        match *self {
            AttachmentType::Output(resource) => resource,
            AttachmentType::Input(resource) => Some(resource)
        }
    }
}

#[derive(Clone, Copy)]
pub struct RenderPassAttachment {
    pub attachment: AttachmentType
}

pub struct RenderPassBuilder<'pass> {
    label: Option<&'pass str>,
    pub colour_attachments: Vec<RenderPassAttachment>,
    pub depth_stencil: Option<RenderPassAttachment>,
}

impl<'pass> RenderPassBuilder<'pass> {
    pub fn render_pass() -> Self {
        RenderPassBuilder {
            label: None,
            colour_attachments: Vec::new(),
            depth_stencil: None
        }
    }

    pub fn label(mut self, label: &'pass str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn add_colour_attachment(mut self, attachment: RenderPassAttachment) -> Self {
        self.colour_attachments.push(attachment);
        self
    }

    pub fn set_depth_stencil_attachment(mut self, depth_stencil: RenderPassAttachment) -> Self {
        self.depth_stencil = Some(depth_stencil);
        self
    }
}
