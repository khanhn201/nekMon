use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use async_graphql::*;

#[derive(SimpleObject, Serialize, Deserialize)]
pub struct Server {
    name: String,
    address: String,
    username: String,
}

#[derive(SimpleObject, Serialize, Deserialize)]
pub struct Project {
    name: String,
    created_at: DateTime<Utc>,
    // files: Vec<String>,
    // runs: Vec<Run>,
}

#[derive(SimpleObject, Serialize, Deserialize)]
pub struct Run {
    id: String,
    name: String,
    notes: String,
    config: String,
    created_at: DateTime<Utc>,

    server: Server,
    remote_directory: String,
    local_directory: String,
    files: Vec<String>,
    
}

#[derive(SimpleObject)]
pub struct Series {
    id: String,
    name: String,
    notes: String,
    config: String,

    server: Server,
    remote_dir: String,
    local_dir: String,
    files: Vec<String>,
}




pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn getProject(
        &self,
        name: String
    ) -> Project {
        Project { name: "test".to_string(), created_at: Utc::now() }
    }
}

impl Project {
    pub fn new() -> Self {
        Project { name: "test".to_string(), created_at: Utc::now() }
    }
}

