use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::{object::Object, resource::Resource};

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
pub struct File {
    pub contents: Vec<Object>,
    pub size: u64,
}

impl File {
    pub fn new(contents: Vec<Object>, size: u64) -> Self {
        File { contents, size }
    }
}

impl<'a> TryFrom<&'a Resource> for &'a File {
    type Error = ();

    fn try_from(value: &'a Resource) -> Result<Self, Self::Error> {
        if let Resource::File(file) = value {
            Ok(file)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Resource> for Arc<File> {
    type Error = ();

    fn try_from(value: Resource) -> Result<Self, Self::Error> {
        if let Resource::File(file) = value {
            Ok(file.clone())
        } else {
            Err(())
        }
    }
}

impl From<Arc<File>> for Resource {
    fn from(val: Arc<File>) -> Self {
        Resource::File(val)
    }
}
