pub mod api {
    pub use super::ResourceManager;
    pub use super::ResourceHandle as Resource;
}

use crate::sparse_set::{ SparseSet, ElementHandle };
use std::collections::{ BinaryHeap, HashMap, HashSet };
use std::time::{ Instant, Duration };
use std::sync::{ Arc, RwLock };
use uuid::Uuid;
use std::borrow::Cow;
use std::path::{ Path, PathBuf };

pub struct ResourceHandle<R> {
    resource_handle: ElementHandle,
    manager: Arc<RwLock<ResourceReferenceManager>>,
    _resource_phantom: std::marker::PhantomData<R>
}

impl<R> ResourceHandle<R> {
    fn new(resource_handle: ElementHandle, manager: Arc<RwLock<ResourceReferenceManager>>) -> ResourceHandle<R> {
        manager.write().unwrap().activate(resource_handle);
        ResourceHandle {
            resource_handle,
            manager,
            _resource_phantom: std::marker::PhantomData
        }
    }
}

impl<R> PartialEq for ResourceHandle<R> {
    fn eq(&self, other: &ResourceHandle<R>) -> bool {
        self.resource_handle.eq(&other.resource_handle)
    }
}
impl<R> Eq for ResourceHandle<R> {}

impl<R> Clone for ResourceHandle<R> {
    fn clone(&self) -> ResourceHandle<R> {
        self.manager.write().unwrap().activate(self.resource_handle);
        ResourceHandle {
            resource_handle: self.resource_handle,
            manager: self.manager.clone(),
            _resource_phantom: std::marker::PhantomData
        }
    }
}

impl<R> std::ops::Drop for ResourceHandle<R> {
    fn drop(&mut self) {
        self.manager.write().unwrap().deactivate(self.resource_handle);
    }
}

pub struct ResourceMetaData<'a> {
    pub uuid: Uuid,
    pub lifetime: ResourceLifetime,
    pub name: Option<Cow<'a, str>>,
    pub path: Option<PathBuf>
}

impl<'s> ResourceMetaData<'s> {
    pub fn new(lifetime: ResourceLifetime) -> ResourceMetaData<'s> {
        ResourceMetaData {
            uuid: Uuid::new_v4(),
            lifetime,
            name: None,
            path: None
        }
    }

    pub fn new_with_name(name: &'static str, lifetime: ResourceLifetime) -> ResourceMetaData<'s> {
        ResourceMetaData {
            uuid: Uuid::new_v4(),
            lifetime,
            name: Some(Cow::Borrowed(name)),
            path: None
        }
    }
}

pub trait ResourceHandler<R> {
    fn create(&mut self, meta_data: &ResourceMetaData) -> R;
    fn destroy(&mut self, resource: R);
}

pub struct ResourceManager<R, H> where
    H: ResourceHandler<R> + Sized {
    last_resource_id: usize,
    resource_id_map: HashMap<Uuid, ElementHandle>,
    name_id_map: HashMap<String, Uuid>,
    path_id_map: HashMap<PathBuf, Uuid>,
    resources: SparseSet<R>,
    resources_being_destroyed: Vec<R>,
    reference_manager: Arc<RwLock<ResourceReferenceManager>>,
    pub handler: H
}

impl<R, H> std::ops::Drop for ResourceManager<R, H> where
    H: ResourceHandler<R> + Sized {
    fn drop(&mut self) {
        for resource_handle in self.resources.get_all_elements() {
            let (_, resource) = self.resources.remove(resource_handle);
            self.handler.destroy(resource.unwrap());
        }
    }
}

impl<R, H> ResourceManager<R, H> where
    H: ResourceHandler<R> + Sized {
    const RESOURCES_TO_DESTROY_PER_UPKEEP: usize = 10;
    pub fn new<const MAX_RESOURCES: usize>(
        handler: H
    ) -> ResourceManager<R, H> {
        let mut resources_being_destroyed = Vec::new();
        resources_being_destroyed.reserve_exact(MAX_RESOURCES);
        ResourceManager {
            last_resource_id: 0,
            resource_id_map: HashMap::new(),
            name_id_map: HashMap::new(),
            path_id_map: HashMap::new(),
            resources: SparseSet::new(MAX_RESOURCES),
            resources_being_destroyed,
            reference_manager: Arc::new(RwLock::new(ResourceReferenceManager::new())),
            handler,
        }
    }

    fn create_resource_handle(&self, element: ElementHandle) -> api::Resource<R> {
        api::Resource::new(element, self.reference_manager.clone())
    }

    pub fn upkeep(&mut self) {
        for resource in self.reference_manager.write().unwrap().upkeep() {
            let (_, resource_dropped) = self.resources.remove(resource);
            // The buffer can be overflowed with mass creation and deletion of objects
            // To avoid moves, we will ensure that we can never overrun the buffer by
            // deleting when the buffer is filled
            if self.resources_being_destroyed.len() == Self::RESOURCES_TO_DESTROY_PER_UPKEEP {
                self.handler.destroy(resource_dropped.unwrap());
            } else {
                self.resources_being_destroyed.push(resource_dropped.unwrap());
            }
        }

        for _ in 0..Self::RESOURCES_TO_DESTROY_PER_UPKEEP.min(self.resources_being_destroyed.len()) {
            let resource = self.resources_being_destroyed.pop().unwrap();
            self.handler.destroy(resource);
        }
    }

    pub fn get_from_path<P: AsRef<Path>>(&self, path: P) -> api::Resource<R> {
        let path_buf = path.as_ref().to_path_buf();
        self.get_from_uuid(self.path_id_map.get(&path_buf).unwrap())
    }

    pub fn get_from_name<N: AsRef<str>>(&self, name: N) -> api::Resource<R> {
        let name_str = name.as_ref().to_string();
        self.get_from_uuid(self.name_id_map.get(&name_str).unwrap())
    }

    pub fn get_from_uuid(&self, uuid: &Uuid) -> api::Resource<R> {
        let resource_id = *self.resource_id_map.get(uuid).unwrap();
        self.create_resource_handle(resource_id)
    }

    pub fn get(&self, resource: &ResourceMetaData) -> api::Resource<R> {
        self.get_from_uuid(&resource.uuid)
    }

    pub fn create(&mut self, meta_resource: &ResourceMetaData) -> api::Resource<R> {
        self.last_resource_id += 1;
        let resource_id = ElementHandle(self.last_resource_id);
        self.resource_id_map.insert(meta_resource.uuid, resource_id);
        self.resources.push(resource_id, self.handler.create(meta_resource));

        if let Some(name) = &meta_resource.name {
            self.name_id_map.insert(name.to_string(), meta_resource.uuid);
        }

        if let Some(path) = &meta_resource.path {
            self.path_id_map.insert(path.to_path_buf(), meta_resource.uuid);
        }

        self.reference_manager.write().unwrap().create(resource_id, meta_resource.lifetime);
        self.create_resource_handle(resource_id)
    }

    pub fn resource(&self, handle: api::Resource<R>) -> &R {
        self.resources.get(handle.resource_handle).unwrap()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
/// How long the resource lasts after all references run out
pub enum ResourceLifetime {
    /// Destroyed immediately
    None,
    /// The resource is unlikely to be loaded again
    Short,
    /// The resource may be loaded again
    Medium,
    /// We expect the resource to be loaded again
    Long,
    /// Never destroyed until the main resource manager is dropped
    Forever
}

struct ResourceReferenceManager {
    all_resources: HashMap<ElementHandle, ResourceReference>,
    active_resources: HashSet<ResourceReference>,
    inactive_resources: BinaryHeap<ResourceReference>
}

impl ResourceReferenceManager {
    const LIFETIMES: [(ResourceLifetime, Duration); 5] = [
        (ResourceLifetime::None, Duration::ZERO),
        (ResourceLifetime::Short, Duration::from_secs(3)),
        (ResourceLifetime::Medium, Duration::from_secs(60)),
        (ResourceLifetime::Long, Duration::from_secs(5 * 60)),
        (ResourceLifetime::Forever, Duration::MAX)
    ];

    fn new() -> ResourceReferenceManager {
        ResourceReferenceManager {
            all_resources: HashMap::new(),
            active_resources: HashSet::new(),
            inactive_resources: BinaryHeap::new()
        }
    }

    fn create(&mut self, resource: ElementHandle, lifetime: ResourceLifetime) {
        if !self.all_resources.contains_key(&resource) {
            self.all_resources.insert(resource, ResourceReference {
                reference_count: 0,
                resource,
                lifetime,
                deletion_time: None
            });
        }
        self.activate(resource);
    }

    fn activate(&mut self, resource: ElementHandle) {
        self.all_resources.get_mut(&resource)
            .expect("Resource must be created before it is activated")
        .reference_count += 1;

        self.active_resources.insert(*self.all_resources.get(&resource).unwrap());
    }

    fn deactivate(&mut self, resource: ElementHandle) {
        self.all_resources.get_mut(&resource)
            .expect("Resource must be created before handle can be dropped")
        .reference_count -= 1;

        if self.all_resources.get(&resource).unwrap().reference_count == 0 {
            self.active_resources.remove(&self.all_resources.get(&resource).unwrap());
            let resource_prototype = self.all_resources.get(&resource).unwrap();
            self.inactive_resources.push(ResourceReference {
                reference_count: resource_prototype.reference_count,
                resource: resource_prototype.resource,
                lifetime: resource_prototype.lifetime,
                deletion_time: Instant::now().checked_add(
                    *Self::LIFETIMES.iter()
                        .find(|(lifetime, _)| *lifetime == resource_prototype.lifetime)
                        .map(|(_, d)| d)
                    .expect("Lifetime not defined")
                )
            });
        }
    }

    fn upkeep(&mut self) -> Vec<ElementHandle> {
        let mut resources_to_delete = Vec::new();
        let now = Instant::now();
        while self.inactive_resources.peek().is_some_and(
            |resource| resource.deletion_time.unwrap() <= now
        ) {
            let resource = self.inactive_resources.peek().unwrap();
            if !self.active_resources.contains(&resource) {
                self.all_resources.remove(&resource.resource);
                resources_to_delete.push(resource.resource);
            }
            self.inactive_resources.pop();
        }

        resources_to_delete
    }
}

#[derive(Clone, Copy)]
struct ResourceReference {
    reference_count: u64,
    resource: ElementHandle,
    lifetime: ResourceLifetime,
    deletion_time: Option<Instant>,
}

impl std::hash::Hash for ResourceReference {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.resource.hash(state);
    }
}

impl PartialEq for ResourceReference {
    fn eq(&self, other: &Self) -> bool {
        self.resource == other.resource
    }
}
impl Eq for ResourceReference {}

impl PartialOrd for ResourceReference {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ResourceReference {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // If a reference doesnt have any deletion time, then it should always be said to
        // be deleted after one with a valid deletion time
        if let None = self.deletion_time {
            std::cmp::Ordering::Greater
        } else if let None = other.deletion_time {
            std::cmp::Ordering::Less
        } else {
            self.deletion_time.cmp(&other.deletion_time)
        }
    }
}

