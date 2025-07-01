#[derive(serde::Deserialize, Debug)]
pub struct TodoistProject {
    pub id: String,
    pub name: String,
}
