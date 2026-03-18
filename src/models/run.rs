use crate::log_parser::Record;
use crate::models::project::Project;
use crate::models::server::Server;
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

    #[serde(default)]
    pub records_json: String,
}

/// ------------------------------
/// Server functions
/// ------------------------------
use leptos::prelude::*;

#[server]
pub async fn get_runs(project_id: i64) -> Result<Vec<Run>, ServerFnError> {
    use crate::app_state::AppState;
    let app_state: AppState =
        use_context::<AppState>().ok_or(ServerFnError::new("expected context"))?;
    let pool = app_state.pool();
    let runs: Vec<Run> = sqlx::query_as("SELECT * FROM run WHERE project_id = ?")
        .bind(project_id)
        .fetch_all(pool)
        .await?;
    Ok(runs)
}

#[server]
pub async fn delete_run(run_id: i64) -> Result<(), ServerFnError> {
    use crate::app_state::AppState;
    let app_state: AppState =
        use_context::<AppState>().ok_or(ServerFnError::new("expected context"))?;
    let pool = app_state.pool();
    sqlx::query("DELETE FROM run WHERE id = ?")
        .bind(run_id)
        .execute(pool)
        .await?;
    Ok(())
}

#[server]
pub async fn update_run(run: Run) -> Result<(), ServerFnError> {
    use crate::app_state::AppState;
    let app_state: AppState =
        use_context::<AppState>().ok_or(ServerFnError::new("expected context"))?;
    let pool = app_state.pool();
    let _run = sqlx::query_as::<_, Run>(
        r#"
        UPDATE run
        SET
            name = COALESCE(?, name),
            remote_directory = COALESCE(?, remote_directory),
            local_directory = COALESCE(?, local_directory),
            post_files = COALESCE(?, post_files),
            get_files = COALESCE(?, get_files),
            config_json = COALESCE(?, config_json),
            notes = COALESCE(?, notes)
        WHERE id = ?
        RETURNING *
        "#,
    )
    .bind(run.name)
    .bind(run.remote_directory)
    .bind(run.local_directory)
    .bind(run.post_files)
    .bind(run.get_files)
    .bind(run.config_json)
    .bind(run.notes)
    .bind(run.id)
    .fetch_one(pool)
    .await?;

    Ok(())
}

#[server]
pub async fn create_run(project_id: i64, server_id: i64) -> Result<(), ServerFnError> {
    use crate::app_state::AppState;
    use rand::distr::{Alphabetic, Alphanumeric, SampleString};
    let app_state: AppState =
        use_context::<AppState>().ok_or(ServerFnError::new("expected context"))?;
    let pool = app_state.pool();
    let name = Alphabetic.sample_string(&mut rand::rng(), 1)
        + &Alphanumeric.sample_string(&mut rand::rng(), 7);
    let project: Project = sqlx::query_as("SELECT * FROM project WHERE id = ?")
        .bind(project_id)
        .fetch_one(pool)
        .await?;
    let server: Server = sqlx::query_as("SELECT * FROM server WHERE id = ?")
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
        r#"INSERT INTO run (name, project_id, server_id, remote_directory, local_directory, post_files, get_files)
         VALUES (?, ?, ?, ?, ?, ?, ?)
         RETURNING *"#
    )
    .bind(name).bind(project_id).bind(server_id).bind(remote_directory).bind(local_directory)
    .bind(project.post_files).bind(project.get_files)
    .fetch_one(pool).await?;

    Ok(run)
}

#[server]
pub async fn download(run_id: i64) -> Result<bool, ServerFnError> {
    use crate::app_state::AppState;
    let app_state: AppState =
        use_context::<AppState>().ok_or(ServerFnError::new("expected context"))?;
    let pool = app_state.pool();
    let servers = app_state.servers();

    let run: Run = sqlx::query_as("SELECT * FROM run WHERE id = ?")
        .bind(run_id)
        .fetch_one(pool)
        .await?;

    let files: Vec<&str> = run
        .get_files
        .split(',')
        .map(|f| f.trim())
        .filter(|f| !f.is_empty())
        .collect();

    for file in &files {
        let local_file = format!("{}/{}", run.local_directory.trim_end_matches('/'), file);
        let remote_file = format!("{}/{}", run.remote_directory.trim_end_matches('/'), file);

        if let Some(parent) = std::path::Path::new(&local_file).parent() {
            std::fs::create_dir_all(parent)?;
        }

        if let Some(ssh_client_ref) = servers.get(&run.server_id) {
            ssh_client_ref
                .download_file(&local_file, &remote_file)
                .await?;
        } else {
            return Ok(false);
        }
    }

    tokio::spawn(async move {
        reparse_and_save(run_id, app_state).await;
    });

    Ok(true)
}

#[cfg(feature = "ssr")]
pub async fn reparse_and_save(run_id: i64, app_state: crate::app_state::AppState) {
    use crate::log_parser::parse;

    let pool = app_state.pool();
    let Ok(run) = sqlx::query_as::<_, Run>("SELECT * FROM run WHERE id = ?")
        .bind(run_id)
        .fetch_one(pool)
        .await
    else {
        return;
    };

    let files: Vec<&str> = run
        .get_files
        .split(',')
        .map(|f| f.trim())
        .filter(|f| !f.is_empty())
        .collect();

    let Some(first) = files.first() else { return };
    let local_file = format!("{}/{}", run.local_directory.trim_end_matches('/'), first);
    let records = parse(&local_file, "default_parser.toml");

    let Ok(json) = serde_json::to_string(&records) else {
        return;
    };

    let _ = sqlx::query("UPDATE run SET records_json = ? WHERE id = ?")
        .bind(json)
        .bind(run_id)
        .execute(pool)
        .await;
}

#[server]
pub async fn get_run_records(run_id: i64) -> Result<Vec<Record>, ServerFnError> {
    use crate::app_state::AppState;
    let app_state = use_context::<AppState>().ok_or(ServerFnError::new("expected context"))?;
    let pool = app_state.pool();

    let run: Run = sqlx::query_as("SELECT * FROM run WHERE id = ?")
        .bind(run_id)
        .fetch_one(pool)
        .await?;

    if run.records_json.is_empty() {
        return Ok(vec![]);
    }

    let mut records: Vec<Record> = serde_json::from_str(&run.records_json).unwrap_or_default();
    records.pop(); // drop in-progress step

    const MAX_POINTS: usize = 5000;
    let len = records.len();

    let records = if len > MAX_POINTS {
        let chunk_size = len / MAX_POINTS;
        records
            .chunks(chunk_size)
            .map(|chunk| {
                let mut avg = Record::new();
                for key in chunk[0].keys() {
                    let mean =
                        chunk.iter().filter_map(|r| r.get(key)).sum::<f64>() / chunk.len() as f64;
                    avg.insert(key.clone(), mean);
                }
                avg
            })
            .collect()
    } else {
        records
    };
    Ok(records)
}
