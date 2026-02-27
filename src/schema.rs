use std::sync::Arc;

use rand::distr::{SampleString,Alphanumeric,Alphabetic};

use async_graphql::*;

use crate::model::*;
use crate::app_state::AppState;
use crate::ssh::SSHClient;



pub struct QueryRoot;
#[Object]
impl QueryRoot {
    async fn get_project(
        &self,
        ctx: &Context<'_>,
        name: String,
    ) -> Result<Project> {
        let app_state = ctx.data::<Arc<AppState>>()?;
        let pool = &app_state.pool;
        let project: Project = sqlx::query_as(
            "SELECT * FROM project WHERE name = ?"
        )
        .bind(name)
        .fetch_one(pool)
        .await?;
        Ok(project)
    }
    async fn get_projects(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<Project>> {
        let app_state = ctx.data::<Arc<AppState>>()?;
        let pool = &app_state.pool;
        let projects: Vec<Project> = sqlx::query_as(
            "SELECT * FROM project"
        )
        .fetch_all(pool)
        .await?;
        Ok(projects)
    }
    async fn get_runs(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<Run>> {
        let app_state = ctx.data::<Arc<AppState>>()?;
        let pool = &app_state.pool;
        let runs: Vec<Run> = sqlx::query_as(
            "SELECT * FROM run"
        )
        .fetch_all(pool)
        .await?;
        Ok(runs)
    }
    async fn get_servers(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<Server>> {
        let app_state = ctx.data::<Arc<AppState>>()?;
        let pool = &app_state.pool;
        let servers: Vec<Server> = sqlx::query_as(
            "SELECT * FROM server"
        )
        .fetch_all(pool)
        .await?;
        Ok(servers)
    }
}


pub struct MutationRoot;
#[Object]
impl MutationRoot {
    async fn create_server(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(regex="^[a-zA-Z][a-zA-Z0-9_-]*$"))]
        name: String,
        address: String,
        username: String,
    ) -> Result<Server> {
        let app_state = ctx.data::<Arc<AppState>>()?;
        let pool = &app_state.pool;
        let server = sqlx::query_as(
            r#"INSERT INTO server (name, address, username)
             VALUES (?, ?, ?)
             RETURNING *"#
        )
        .bind(&name).bind(&address).bind(&username)
        .fetch_one(pool).await?;
        Ok(server)
    }
    async fn create_project(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(regex="^[a-zA-Z][a-zA-Z0-9_-]*$"))]
        name: String,
    ) -> Result<Project> {
        let app_state = ctx.data::<Arc<AppState>>()?;
        let pool = &app_state.pool;
        let project = sqlx::query_as(
            r#"INSERT INTO project (name)
             VALUES (?)
             RETURNING *"#
        )
        .bind(&name)
        .fetch_one(pool).await?;

        Ok(project)
    }
    async fn create_run(
        &self,
        ctx: &Context<'_>,
        project_id: i64,
        server_id: i64,
    ) -> Result<Run> {
        let app_state = ctx.data::<Arc<AppState>>()?;
        let pool = &app_state.pool;
        let name = Alphabetic.sample_string(&mut rand::rng(), 1) + &Alphanumeric.sample_string(&mut rand::rng(), 7);
        
        let project: Project = sqlx::query_as(
            "SELECT * FROM project WHERE id = ?"
        )
        .bind(project_id)
        .fetch_one(pool)
        .await?;
        let server: Server = sqlx::query_as(
            "SELECT * FROM server WHERE id = ?"
        )
        .bind(server_id)
        .fetch_one(pool)
        .await?;
        // TODO: error if directory not set
        let remote_directory = format!(
            "{}/{}/{}/",
            server.remote_directory.trim_end_matches('/'),
            project.name,
            name
        );
        let local_directory = format!(
            "{}/{}/",
            project.local_directory.trim_end_matches('/'),
            name
        );

        let run = sqlx::query_as(
            r#"INSERT INTO run (name, project_id, server_id, remote_directory, local_directory, post_files_json, get_files_json)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             RETURNING *"#
        )
        .bind(name).bind(project_id).bind(server_id).bind(remote_directory).bind(local_directory)
        .bind(project.post_files_json).bind(project.get_files_json)
        .fetch_one(pool).await?;

        Ok(run)
    }

    async fn update_server(
        &self,
        ctx: &Context<'_>,
        id: i64,
        #[graphql(validator(regex="^[a-zA-Z][a-zA-Z0-9_-]*$"))]
        name: Option<String>,
        address:  Option<String>,
        username:  Option<String>,
        remote_directory:  Option<String>,
        key_file_path: Option<String>,
        port: Option<u16>,
    ) -> Result<Server> {
        let app_state = ctx.data::<Arc<AppState>>()?;
        let pool = &app_state.pool;
        // TODO: error if directory not valid

        let server = sqlx::query_as::<_, Server>(
            r#"
            UPDATE server
            SET
                name = COALESCE(?, name),
                address = COALESCE(?, address),
                username = COALESCE(?, username),
                remote_directory = COALESCE(?, remote_directory),
                key_file_path = COALESCE(?, key_file_path),
                port = COALESCE(?, port)
            WHERE id = ?
            RETURNING *
            "#
        )
        .bind(name).bind(address).bind(username).bind(remote_directory)
        .bind(key_file_path).bind(port).bind(id)
        .fetch_one(pool)
        .await?;

        Ok(server)
    }

    async fn update_project(
        &self,
        ctx: &Context<'_>,
        id: i64,
        #[graphql(validator(regex="^[a-zA-Z][a-zA-Z0-9_-]*$"))]
        name: Option<String>,
        src_directory: Option<String>,
        local_directory: Option<String>,
        post_file_json: Option<String>,
        get_file_json: Option<String>,
    ) -> Result<Project> {
        let app_state = ctx.data::<Arc<AppState>>()?;
        let pool = &app_state.pool;

        let project = sqlx::query_as::<_, Project>(
            r#"
            UPDATE project
            SET
                name = COALESCE(?, name),
                src_directory = COALESCE(?, src_directory),
                local_directory = COALESCE(?, local_directory),
                post_files_json = COALESCE(?, post_files_json),
                get_files_json = COALESCE(?, get_files_json)
            WHERE id = ?
            RETURNING *
            "#
        )
        .bind(name).bind(src_directory).bind(local_directory)
        .bind(post_file_json).bind(get_file_json).bind(id)
        .fetch_one(pool)
        .await?;

        Ok(project)
    }

    async fn update_run(
        &self,
        ctx: &Context<'_>,
        id: i64,
        #[graphql(validator(regex="^[a-zA-Z][a-zA-Z0-9_-]*$"))]
        name: Option<String>,
        remote_directory:  Option<String>,
        local_directory:  Option<String>,
        post_file_json: Option<String>,
        get_file_json: Option<String>,
        config: Option<String>,
        notes: Option<String>,
    ) -> Result<Run> {
        let app_state = ctx.data::<Arc<AppState>>()?;
        let pool = &app_state.pool;

        let run = sqlx::query_as::<_, Run>(
            r#"
            UPDATE run
            SET
                name = COALESCE(?, name),
                remote_directory = COALESCE(?, remote_directory),
                local_directory = COALESCE(?, local_directory),
                post_files_json = COALESCE(?, post_files_json),
                get_files_json = COALESCE(?, get_files_json),
                config_json = COALESCE(?, config_json),
                notes = COALESCE(?, notes)
            WHERE id = ?
            RETURNING *
            "#
        )
        .bind(name).bind(remote_directory).bind(local_directory)
        .bind(post_file_json).bind(get_file_json).bind(config)
        .bind(notes).bind(id)
        .fetch_one(pool)
        .await?;

        Ok(run)
    }

}

// Additional resolvers

#[ComplexObject]
impl Project {
    async fn runs(&self, ctx: &Context<'_>) -> Result<Vec<Run>> {
        let app_state = ctx.data::<Arc<AppState>>()?;
        let pool = &app_state.pool;

        let runs = sqlx::query_as::<_, Run>(
            "SELECT * FROM run WHERE project_id = ?"
        )
        .bind(self.id)
        .fetch_all(pool)
        .await?;

        Ok(runs.into_iter().map(Run::from).collect())
    }
}

#[ComplexObject]
impl Server {
    async fn alive(&self, ctx: &Context<'_>) -> Result<bool> {
        let app_state = ctx.data::<Arc<AppState>>()?;
        let servers = &app_state.servers;
        
        if let Some(ssh_client_ref) = servers.get(&self.id) {
            return match ssh_client_ref.ping().await.is_ok() {
                true => Ok(true),
                false => {
                    servers.remove(&self.id);
                    return Ok(false);
                }
            }
        }

        match SSHClient::new(self).await {
            Ok(client) => {
                servers.insert(self.id, Arc::new(client));
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }
}
