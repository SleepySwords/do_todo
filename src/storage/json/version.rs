use serde::{Deserialize, Serialize};

use crate::task::TaskStore;

#[derive(Deserialize, Serialize)]
#[serde(tag = "version")]
pub enum JSONVersion {
    #[serde(rename = "0")]
    V0(TaskStore),
    // #[serde(other)]
    // Unknown,
}

impl Default for JSONVersion {
    fn default() -> Self {
        JSONVersion::V0(TaskStore::default())
    }
}

impl From<JSONVersion> for TaskStore {
    fn from(value: JSONVersion) -> Self {
        match value {
            JSONVersion::V0(store) => store,
        }
    }
}
