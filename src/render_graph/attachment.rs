use uuid::Uuid;
pub use crate::render_graph::handle_map::Handle as AttachmentHandle;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Id {
    Uuid(Uuid),
    String(String)
}

#[derive(Debug, Copy, Clone)]
pub enum Type {

}

#[derive(Debug, Clone)]
pub struct Attachment {
    attachment_type: Type,
    id: Id
}

impl Attachment {
}

