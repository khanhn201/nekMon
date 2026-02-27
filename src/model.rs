use time::OffsetDateTime;
use serde::{Serialize, Deserialize};

use async_graphql::*;

use sqlx::FromRow;


// Models: SQL table schema

#[derive(SimpleObject, Serialize, Deserialize, FromRow)]
#[graphql(complex)]
pub struct Server {
    pub id: i64,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub username: String,
    pub key_file_path: String,
    //  !TODO a run script, can be different for Nek5000 and NekRS?
    
    pub remote_directory: String, // Default prefix for each new run
    // pub run_script: String // TODO
}

#[derive(SimpleObject, Serialize, Deserialize, FromRow)]
#[graphql(complex)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub created_at: OffsetDateTime,
    
    pub local_directory: String,  // Default prefix for each new run
    //  TODO an initial script to generate mesh, parameters, etc.

    pub src_directory: String,
    pub post_files_json: String,   // JSON of a list of files to copy to server
    pub get_files_json: String,    // JSON of a list of files to retrieve from server
}

#[derive(SimpleObject, Serialize, Deserialize, FromRow)]
pub struct Run {
    pub id: i64,
    pub name: String,
    pub created_at: OffsetDateTime,
    pub project_id: i64,
    pub server_id: i64,

    pub remote_directory: String,
    pub local_directory: String,

    pub post_files_json: String,   // JSON of a list of files to copy to server
    pub get_files_json: String,    // JSON of a list of files to retrieve from server
    pub config_json: String,
    pub notes: String,
}
