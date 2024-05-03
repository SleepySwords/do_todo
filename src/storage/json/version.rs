use serde::{Deserialize, Serialize};

use crate::{data::json_data_store::JsonDataStore, task::TaskStore};

#[derive(Deserialize, Serialize)]
#[serde(tag = "version")]
pub enum JSONVersion {
    #[serde(rename = "0")]
    V0(TaskStore),
    #[serde(rename = "1")]
    V1(JsonDataStore),
    // #[serde(other)]
    // Unknown,
}

impl Default for JSONVersion {
    fn default() -> Self {
        JSONVersion::V0(TaskStore::default())
    }
}

impl From<JSONVersion> for JsonDataStore {
    fn from(value: JSONVersion) -> Self {
        match value {
            JSONVersion::V0(store) => todo!(),
            JSONVersion::V1(store) => store,
        }
    }
}
