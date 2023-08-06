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
    handle_map: HashMap<HandleT, T>
}

impl<T, HandleT> HandleMap<HandleT, T> where 
    HandleT: HandleType + Copy + std::hash::Hash + PartialEq + Eq {
    pub fn new() -> Self {
        HandleMap {
            string_map: HashMap::new(),
            handle_map: HashMap::new()
        }
    }

    pub fn add(&mut self, object: T, string_id: Option<String>) -> HandleT {
        let handle = HandleT::new();
        self.handle_map.insert(handle, object);
        if let Some(id) = string_id {
            self.string_map.insert(id, handle);
        }
        handle
    }

    pub fn get_from_string(&self, string_id: &String) -> Option<&T> {
        if !self.string_map.contains_key(string_id) {
            return None
        }
        let handle = self.string_map.get(string_id).unwrap();
        self.get_from_handle(&handle)
    }

    pub fn get_from_handle(&self, handle: &HandleT) -> Option<&T> {
        self.handle_map.get(handle)
    }
}
