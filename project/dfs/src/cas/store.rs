use std::{
    io::{self, Error},
    sync::Arc,
};

use libc::EIO;

use super::{
    chunk::{Chunk, CHUNK_SIZE},
    directory::Directory,
    error::PathResolutionError,
    file::File,
    object::Object,
    resource::Resource,
};

pub trait ContentAddressedStore {
    fn get_resource(&self, object: &Object) -> Option<Resource>;
    fn has(&self, object: &Object) -> bool;
    fn accessible_objects(&self) -> Result<Vec<Object>, PathResolutionError>;

    fn get<T>(&self, object: &Object) -> Option<T>
    where
        T: TryFrom<Resource>,
    {
        let resource: Resource = self.get_resource(object)?.clone();
        resource.try_into().ok()
    }

    fn read_file(&self, file: Arc<File>, offset: u64, size: u64) -> io::Result<Vec<u8>> {
        let end_location = (offset + size).min(file.size) as usize;
        let start_chunk = offset.div_floor(CHUNK_SIZE) as usize;
        let end_chunk = end_location.div_ceil(CHUNK_SIZE as usize);
        let chunk_contents = file.contents[start_chunk..end_chunk].iter().map(|o| {
            let chunk: Arc<Chunk> = self.get(o).ok_or(Error::from_raw_os_error(EIO))?;
            Ok(chunk.data.clone())
        });
        let chunk_data = chunk_contents.collect::<Result<Vec<Vec<u8>>, Error>>()?;
        let chunk_data = chunk_data.iter().flatten().cloned().collect::<Vec<u8>>();

        let vec_start = start_chunk * CHUNK_SIZE as usize;
        let start_chunk_offset = (offset as usize) - vec_start;
        let end_offset = end_location - vec_start;
        Ok(chunk_data[start_chunk_offset..end_offset].to_vec())
    }

    fn resolve_path(&self, path: &str) -> Result<Object, PathResolutionError> {
        let mut path_components = path.split('/');

        if !matches!(path_components.next(), Some("")) {
            return Err(PathResolutionError::new("paths must begin with '/'"));
        }

        let root_hash_hex = path_components
            .next()
            .ok_or(PathResolutionError::new("missing root hash"))?;
        let mut root_hash = [0; 32];
        hex::decode_to_slice(root_hash_hex, &mut root_hash)
            .map_err(|_| PathResolutionError::new("root name is not a valid sha256 hash"))?;

        let root_object: Object = Object::new(root_hash);

        let root: Arc<Directory> = self
            .get(&root_object)
            .ok_or(PathResolutionError::new("unable to find root"))?;
        let mut curr_dir = root;
        let mut curr_item = root_object;

        while let Some(component) = path_components.next() {
            let item = curr_dir.get_child(component)?;
            let resource = self.get(&item).ok_or(PathResolutionError::new(&format!(
                "unable to find '{component}'"
            )))?;

            match resource {
                Resource::Directory(dir) => {
                    curr_dir = dir;
                }
                Resource::File(_) => {
                    // this consumes the iterator but that's fine because we return here
                    return match path_components.next() {
                        None => Ok(item),
                        _ => Err(PathResolutionError::new(&format!(
                            "unable to find '{component}'"
                        ))),
                    };
                }
                Resource::Chunk(_) => {}
            }

            curr_item = item;
        }

        Ok(curr_item)
    }
}
