use std::ops::Deref;

use serde::{Deserialize, Serialize};

/// XXH3 Hash
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct XXH3Hash(u64);

impl XXH3Hash {
    pub fn new(hash: u64) -> Self {
        Self(hash)
    }
}

impl Deref for XXH3Hash {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Debug for XXH3Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "XXH3Hash({:016x})", self.0)
    }
}
