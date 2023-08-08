use uuid::Uuid;
pub use crate::render_graph::handle_map::Handle as ResourceHandle;

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
pub enum Resource<'resource> {
    Persistent(Id<'resource>),
    Dynamic(Uuid)
}

impl<'a> Resource<'a> {
    pub fn into_persistent(&self) -> Resource<'a> {
        match self {
            Resource::Persistent(id) => Resource::Persistent(*id),
            Resource::Dynamic(uuid) => Resource::Persistent(Id {
                global_id: *uuid,
                string_id: None
            })
        }
    }
}

