use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use crate::cas::{
    chunk::Chunk, directory::Directory, error::PathResolutionError, file::File, object::Object,
    resource::Resource, ContentAddressedStore,
};

#[derive(Default, Debug)]
pub struct LocalStore {
    resources: BTreeMap<Object, Resource>,
}

impl LocalStore {
    pub fn new() -> Self {
        Self {
            resources: BTreeMap::new(),
        }
    }

    pub fn add_resource(&mut self, resource: Resource) {
        let object = Object::from(&resource);
        self.resources.entry(object).or_insert(resource);
    }

    pub fn create_file(&mut self, contents: &[u8]) -> Object {
        let chunks = Chunk::chunks_from_data(contents);
        let chunk_objects = chunks.into_iter().map(|chunk| {
            let arc = Arc::new(chunk);
            let chunk_object = Object::from(&arc.clone().into());

            self.resources.entry(chunk_object).or_insert(arc.into());

            chunk_object
        });

        let file = File::new(chunk_objects.collect(), contents.len() as u64);
        let resource: Resource = Arc::new(file).into();
        let object = Object::from(&resource);
        self.resources.entry(object).or_insert(resource);

        object
    }

    pub fn create_directory(&mut self, contents: BTreeMap<String, Object>) -> Object {
        let dir = Directory::new(contents);
        let resource: Resource = Arc::new(dir).into();
        let object = Object::from(&resource);
        self.resources.entry(object).or_insert(resource);

        object
    }
}

impl ContentAddressedStore for LocalStore {
    fn get_resource(&self, object: &Object) -> Option<Resource> {
        self.resources.get(object).cloned()
    }

    fn has(&self, object: &Object) -> bool {
        self.resources.contains_key(object)
    }

    fn accessible_objects(&self) -> Result<Vec<Object>, PathResolutionError> {
        let accessible_objects = self
            .resources
            .iter()
            .filter_map(|(obj, resource)| {
                let file: Result<&File, ()> = resource.try_into();
                let directory: Result<&Directory, ()> = resource.try_into();

                if file.is_ok() || directory.is_ok() {
                    Some(*obj)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        Ok(accessible_objects)
    }
}

impl ContentAddressedStore for Arc<Mutex<LocalStore>> {
    fn get_resource(&self, object: &Object) -> Option<Resource> {
        self.lock().unwrap().get(object)
    }

    fn has(&self, object: &Object) -> bool {
        self.lock().unwrap().has(object)
    }

    fn accessible_objects(&self) -> Result<Vec<Object>, PathResolutionError> {
        self.lock().unwrap().accessible_objects()
    }
}
