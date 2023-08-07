use uuid::Uuid;
pub use crate::render_graph::handle_map::Handle as AttachmentHandle;

#[derive(Debug, Copy, Clone)]
pub struct Id<'id> {
    pub global_id: Uuid,
    pub string_id: Option<&'id str>
}

impl Id<'_> {
    pub fn new<'a>() -> Id<'a> {
        Id {
            global_id: Uuid::new_v4(),
            string_id: None
        }
    }

    pub fn new_with_name<'a>(name: &'a str) -> Id<'a> {
        Id {
            global_id: Uuid::new_v4(),
            string_id: Some(name)
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Attachment<'attachment> {
    Persistent(Id<'attachment>),
    Dynamic(Uuid)
}

impl<'a> Attachment<'a> {
    pub fn into_persistent(&self) -> Attachment<'a> {
        match self {
            Attachment::Persistent(id) => Attachment::Persistent(*id),
            Attachment::Dynamic(uuid) => Attachment::Persistent(Id {
                global_id: *uuid,
                string_id: None
            })
        }
    }
}

