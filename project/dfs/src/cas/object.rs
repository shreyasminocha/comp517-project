use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

use hex::ToHex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::resource::Resource;

/// A hashed resource.
#[derive(Serialize, Deserialize)]
pub struct Object {
    #[serde(with = "serde_arrays")]
    pub hash: [u8; 32],
}

impl Object {
    pub fn new(hash: [u8; 32]) -> Self {
        Object { hash }
    }
}

impl From<&Resource> for Object {
    fn from(value: &Resource) -> Self {
        let serialized = serde_json::to_string(value).unwrap();

        let mut hasher = Sha256::new();
        hasher.update(serialized);
        let hash: [u8; 32] = hasher
            .finalize()
            .as_slice()
            .try_into()
            .expect("sha256 hash should have 256 bits");

        Object::new(hash)
    }
}

impl Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for Object {}

impl PartialOrd for Object {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Object {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hash.cmp(&other.hash)
    }
}

impl Copy for Object {}

impl Clone for Object {
    fn clone(&self) -> Self {
        *self
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.hash.encode_hex::<String>())
    }
}

impl Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Object")
            .field("hash", &self.hash.encode_hex::<String>())
            .finish()
    }
}
