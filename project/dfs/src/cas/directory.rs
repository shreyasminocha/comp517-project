use std::collections::BTreeMap;
use std::fmt::{self, Display};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::error::{FilesystemError, PathResolutionError};
use super::object::Object;
use super::resource::Resource;

#[derive(PartialEq, Eq, Clone, Hash, Serialize, Deserialize, Debug)]
pub struct DirectoryEntry {
    pub name: String,
    pub file: Object,
}

impl Display for DirectoryEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.name, self.file)
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
pub struct Directory {
    contents: BTreeMap<String, Object>,
}

impl Directory {
    pub fn new(contents: BTreeMap<String, Object>) -> Self {
        Directory { contents }
    }

    pub fn add_resource(&self, entry: DirectoryEntry) -> Result<Directory, FilesystemError> {
        if !self.contents.contains_key(&entry.name) {
            return Err(FilesystemError::new(&format!(
                "resource with name '{}' already exists",
                entry.name
            )));
        }

        let mut new_contents = self.contents.clone();
        new_contents.insert(entry.name, entry.file);

        let new_directory = Directory {
            contents: new_contents,
        };

        Ok(new_directory)
    }

    pub fn replace_resource(
        &self,
        old_resource: Object,
        new_entry: DirectoryEntry,
    ) -> Result<Directory, FilesystemError> {
        let mut new_contents = self.contents.clone();
        new_contents.retain(|_, file| *file != old_resource);
        new_contents.insert(new_entry.name, new_entry.file);

        let new_directory = Directory {
            contents: new_contents,
        };

        Ok(new_directory)
    }

    pub fn get_children(&self) -> impl Iterator<Item = DirectoryEntry> + '_ {
        self.contents.iter().map(|(name, file)| DirectoryEntry {
            name: name.clone(), // TODO: erm
            file: *file,
        })
    }

    pub fn get_child(&self, name: &str) -> Result<Object, PathResolutionError> {
        if name.is_empty() {
            // todo: move this check elsewhere so we don't have to make a copy here
            let res: Resource = Arc::new(self.clone()).into();
            return Ok(Object::from(&res));
        }

        let child = self.contents.get(name);
        match child {
            Some(item) => Ok(*item),
            None => Err(PathResolutionError::new(&format!(
                "unable to find '{name}'",
            ))),
        }
    }
}

impl Default for Directory {
    fn default() -> Self {
        Self::new(BTreeMap::new())
    }
}

impl<'a> TryFrom<&'a Resource> for &'a Directory {
    type Error = ();

    fn try_from(value: &'a Resource) -> Result<Self, Self::Error> {
        if let Resource::Directory(directory) = value {
            Ok(directory)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Resource> for Arc<Directory> {
    type Error = ();

    fn try_from(value: Resource) -> Result<Self, Self::Error> {
        if let Resource::Directory(directory) = value {
            Ok(directory)
        } else {
            Err(())
        }
    }
}

impl From<Arc<Directory>> for Resource {
    fn from(val: Arc<Directory>) -> Self {
        Resource::Directory(val)
    }
}
