use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use async_graphql::*;

fn default_time() -> OffsetDateTime {
    // ALlow missing field on create
    OffsetDateTime::UNIX_EPOCH
}

// Models: Should be one-to-one with SQL tables

#[derive(SimpleObject, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[cfg_attr(feature = "ssr", graphql(complex))]
pub struct Server {
    #[serde(default)]
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

#[derive(SimpleObject, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[cfg_attr(feature = "ssr", graphql(complex))]
pub struct Project {
    #[serde(default)]
    pub id: i64,
    pub name: String,
    #[serde(default = "default_time")]
    pub created_at: OffsetDateTime,

    pub local_directory: String, // Default prefix for each new run
    //  TODO an initial script to generate mesh, parameters, etc.
    pub src_directory: String,
    pub post_files: String, // comma separated of files to copy to server
    pub get_files: String,  // comma separated of files to retrieve from server
}

#[derive(SimpleObject, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[cfg_attr(feature = "ssr", graphql(complex))]
pub struct Run {
    #[serde(default)]
    pub id: i64,
    pub name: String,
    #[serde(default = "default_time")]
    pub created_at: OffsetDateTime,
    pub project_id: i64,
    pub server_id: i64,

    pub remote_directory: String,
    pub local_directory: String,

    pub post_files: String, // comma separated list of files to copy to server
    pub get_files: String,  // comma separated list of files to retrieve from server
    pub config_json: String,
    pub notes: String,
}
