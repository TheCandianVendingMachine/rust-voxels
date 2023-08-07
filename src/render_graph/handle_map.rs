use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Handle(Uuid);

pub trait HandleType {
    fn new() -> Self;
    fn uuid(&self) -> Uuid;
}

impl HandleType for Handle where {
    fn new() -> Self {
        Handle(Uuid::new_v4())
    }

    fn uuid(&self) -> Uuid {
        self.0
    }
}

pub struct HandleMap<HandleT, T> 
    where HandleT: HandleType + Copy + std::hash::Hash + PartialEq + Eq  {
    string_map: HashMap<String, HandleT>,
    handle_map: HashMap<HandleT, T>,
    handle_to_string_map: HashMap<HandleT, String>
}

impl<T, HandleT> HandleMap<HandleT, T> where 
    HandleT: HandleType + Copy + std::hash::Hash + PartialEq + Eq {
    pub fn new() -> Self {
        HandleMap {
            string_map: HashMap::new(),
            handle_map: HashMap::new(),
            handle_to_string_map: HashMap::new()
        }
    }

    pub fn add(&mut self, object: T, string_id: Option<String>) -> HandleT {
        let handle = HandleT::new();
        self.handle_map.insert(handle, object);
        if let Some(id) = string_id {
            self.string_map.insert(id.clone(), handle);
            self.handle_to_string_map.insert(handle, id);
        }
        handle
    }

    pub fn get_from_string(&self, string_id: &String) -> Option<&T> {
        self.string_map.get(string_id).map_or(None, |h| self.get_from_handle(h))
    }

    pub fn get_from_handle(&self, handle: &HandleT) -> Option<&T> {
        self.handle_map.get(handle)
    }

    pub fn get_string_from_handle(&self, handle: &HandleT) -> Option<String> {
        self.handle_to_string_map.get(handle).map(|s| s.clone())
    }
}
