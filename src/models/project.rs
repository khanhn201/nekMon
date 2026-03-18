use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// ------------------------------
/// Struct: one-to-one with SQL table
/// ------------------------------

fn default_time() -> OffsetDateTime {
    // ALlow missing field on create
    OffsetDateTime::UNIX_EPOCH
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
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

/// ------------------------------
/// Server functions
/// ------------------------------
use leptos::prelude::*;

#[server]
pub async fn get_projects() -> Result<Vec<Project>, ServerFnError> {
    use crate::app_state::AppState;
    let app_state: AppState =
        use_context::<AppState>().ok_or(ServerFnError::new("expected context"))?;
    let pool = app_state.pool();
    let projects: Vec<Project> = sqlx::query_as("SELECT * FROM project")
        .fetch_all(pool)
        .await?;
    Ok(projects)
}

#[server]
pub async fn create_project(project: Project) -> Result<(), ServerFnError> {
    use crate::app_state::AppState;
    let app_state: AppState =
        use_context::<AppState>().ok_or(ServerFnError::new("expected context"))?;
    let pool = app_state.pool();
    let _project = sqlx::query_as::<_, Project>(
        r#"INSERT INTO project (name, src_directory, local_directory, post_files, get_files)
         VALUES (?, ?, ?, ?, ?)
         RETURNING *"#,
    )
    .bind(project.name)
    .bind(project.src_directory)
    .bind(project.local_directory)
    .bind(project.post_files)
    .bind(project.get_files)
    .fetch_one(pool)
    .await?;
    Ok(())
}

#[server]
pub async fn update_project(project: Project) -> Result<(), ServerFnError> {
    use crate::app_state::AppState;
    let app_state: AppState =
        use_context::<AppState>().ok_or(ServerFnError::new("expected context"))?;
    let pool = app_state.pool();
    let _project = sqlx::query_as::<_, Project>(
        r#"
        UPDATE project
        SET
            name = COALESCE(?, name),
            src_directory = COALESCE(?, src_directory),
            local_directory = COALESCE(?, local_directory),
            post_files = COALESCE(?, post_files),
            get_files = COALESCE(?, get_files)
        WHERE id = ?
        RETURNING *
        "#,
    )
    .bind(project.name)
    .bind(project.src_directory)
    .bind(project.local_directory)
    .bind(project.post_files)
    .bind(project.get_files)
    .bind(project.id)
    .fetch_one(pool)
    .await?;

    Ok(())
}

#[server]
pub async fn delete_project(project_id: i64) -> Result<(), ServerFnError> {
    use crate::app_state::AppState;
    let app_state: AppState =
        use_context::<AppState>().ok_or(ServerFnError::new("expected context"))?;
    let pool = app_state.pool();
    sqlx::query("DELETE FROM project WHERE id = ?")
        .bind(project_id)
        .execute(pool)
        .await?;
    Ok(())
}
