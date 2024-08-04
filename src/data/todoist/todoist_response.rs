use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct TodoistResponse {
    pub temp_id_mapping: HashMap<String, String>,
    sync_status: HashMap<String, String>, //FIXME Not for errors tho!
}
