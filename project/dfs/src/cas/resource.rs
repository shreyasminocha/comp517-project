use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::{chunk::Chunk, directory::Directory, file::File};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Resource {
    Chunk(Arc<Chunk>),
    File(Arc<File>),
    Directory(Arc<Directory>),
}
