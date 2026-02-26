use rand::distr::{SampleString,Alphanumeric};

use time::OffsetDateTime;

use async_graphql::*;

use sqlx::{SqlitePool};

use crate::model::*;


#[derive(InputObject)]
pub struct UpdateProjectInput {
    pub id: i64,
    #[graphql(validator(regex="^[a-zA-Z0-9_-]+$"))]
    pub name: Option<String>,
    pub src_directory: Option<String>,
    pub local_directory: Option<String>,
    pub post_file_json: Option<String>,
    pub get_file_json: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateServerInput {
    pub id: i64,
    pub name: Option<String>,
    pub address:  Option<String>,
    pub username:  Option<String>,
    pub remote_directory:  Option<String>,
}

#[derive(InputObject)]
pub struct UpdateRunInput {
    pub id: i64,
    #[graphql(validator(regex="^[a-zA-Z0-9_-]+$"))]
    pub name: Option<String>,
    pub remote_directory:  Option<String>,
    pub local_directory:  Option<String>,
    pub post_file_json: Option<String>,
    pub get_file_json: Option<String>,
    pub config: Option<String>,
    pub notes: Option<String>,
}




pub struct QueryRoot;
#[Object]
impl QueryRoot {
    async fn get_project(
        &self,
        ctx: &Context<'_>,
        name: String,
    ) -> Result<Project> {
        let pool = ctx.data_unchecked::<SqlitePool>();
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
        let pool = ctx.data_unchecked::<SqlitePool>();
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
        let pool = ctx.data_unchecked::<SqlitePool>();
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
        let pool = ctx.data_unchecked::<SqlitePool>();
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
        name: String,
        address: String,
        username: String,
    ) -> Result<Server> { // TODO: Allow alias
        let pool = ctx.data_unchecked::<SqlitePool>();
        let server = sqlx::query_as(
            "INSERT INTO server (name, address, username, remote_directory)
             VALUES (?, ?, ?, '')
             RETURNING *"
        )
        .bind(&name).bind(&address).bind(&username)
        .fetch_one(pool).await?;
        Ok(server)
    }
    async fn create_project(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(regex="^[a-zA-Z0-9_-]+$"))] name: String,
    ) -> Result<Project> {
        let pool = ctx.data_unchecked::<SqlitePool>();
        let created_at = OffsetDateTime::now_utc();
        let project = sqlx::query_as(
            "INSERT INTO project (name, created_at, local_directory, src_directory, post_files_json, get_files_json)
             VALUES (?, ?, '', '', '', '')
             RETURNING *"
        )
        .bind(&name).bind(created_at)
        .fetch_one(pool).await?;

        Ok(project)
    }
    async fn create_run(
        &self,
        ctx: &Context<'_>,
        project_id: i64,
        server_id: i64,
    ) -> Result<Run> {
        let pool = ctx.data_unchecked::<SqlitePool>();
        let name = Alphanumeric.sample_string(&mut rand::rng(), 8);

        let created_at = OffsetDateTime::now_utc();
        
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
        
        
        let remote_directory = format!(
            "{}/{}/{}",
            server.remote_directory.trim_end_matches('/'),
            project.name,
            name
        );

        let local_directory = format!(
            "{}/{}",
            project.local_directory.trim_end_matches('/'),
            name
        );

        let run = sqlx::query_as(
            "INSERT INTO run (name, created_at, project_id, server_id, remote_directory, local_directory, post_files_json, get_files_json, config_json, notes)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, '', '')
             RETURNING *"
        )
        .bind(name).bind(created_at).bind(project_id).bind(server_id).bind(remote_directory).bind(local_directory)
        .bind(project.post_files_json).bind(project.get_files_json)
        .fetch_one(pool).await?;

        Ok(run)
    }

    async fn update_server(
        &self,
        ctx: &Context<'_>,
        input: UpdateServerInput,
    ) -> Result<Server> {
        let pool = ctx.data_unchecked::<SqlitePool>();

        let server = sqlx::query_as::<_, Server>(
            r#"
            UPDATE server
            SET
                name = COALESCE(?, name),
                address = COALESCE(?, address),
                username = COALESCE(?, username),
                remote_directory = COALESCE(?, remote_directory)
            WHERE id = ?
            RETURNING *
            "#
        )
        .bind(input.name)
        .bind(input.address)
        .bind(input.username)
        .bind(input.remote_directory)
        .bind(input.id)
        .fetch_one(pool)
        .await?;

        Ok(server)
    }

    async fn update_project(
        &self,
        ctx: &Context<'_>,
        input: UpdateProjectInput,
    ) -> Result<Project> {
        let pool = ctx.data_unchecked::<SqlitePool>();

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
        .bind(input.name)
        .bind(input.src_directory)
        .bind(input.local_directory)
        .bind(input.post_file_json)
        .bind(input.get_file_json)
        .bind(input.id)
        .fetch_one(pool)
        .await?;

        Ok(project)
    }

    async fn update_run(
        &self,
        ctx: &Context<'_>,
        input: UpdateRunInput,
    ) -> Result<Run> {
        let pool = ctx.data_unchecked::<SqlitePool>();

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
        .bind(input.name)
        .bind(input.remote_directory)
        .bind(input.local_directory)
        .bind(input.post_file_json)
        .bind(input.get_file_json)
        .bind(input.config)
        .bind(input.notes)
        .bind(input.id)
        .fetch_one(pool)
        .await?;

        Ok(run)
    }

}
