use hex::ToHex;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, sync::Arc};

use super::resource::Resource;

pub const CHUNK_SIZE: u64 = 1024 * 1024 * 4;

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct Chunk {
    pub data: Vec<u8>,
}

impl Chunk {
    pub fn chunks_from_data(data: &[u8]) -> Vec<Chunk> {
        data[..]
            .chunks(CHUNK_SIZE as usize)
            .map(|d| Chunk { data: d.to_vec() })
            .collect()
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunk")
            .field("data", &self.data.encode_hex::<String>())
            .finish()
    }
}

impl TryFrom<Resource> for Arc<Chunk> {
    type Error = ();

    fn try_from(value: Resource) -> Result<Self, Self::Error> {
        if let Resource::Chunk(chunk) = value {
            Ok(chunk)
        } else {
            Err(())
        }
    }
}

impl From<Arc<Chunk>> for Resource {
    fn from(val: Arc<Chunk>) -> Self {
        Resource::Chunk(val)
    }
}
