use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NewTag {
    pub name: String,
    pub color: Option<String>,
}
