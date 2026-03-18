use leptos::prelude::*;
use leptos_router::components::A;

use lucide_leptos::{Ellipsis, Plus, Dot};

use crate::components::modal::{Menu, Modal};
use crate::model::{Run, Project, Server};
use crate::components::server_list::get_servers;
use crate::log_parser::Record;


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
async fn download(run_id: i64) -> Result<bool, ServerFnError> {
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
async fn reparse_and_save(run_id: i64, app_state: crate::app_state::AppState) {
    use crate::log_parser::parse;

    let pool = app_state.pool();
    let Ok(run) = sqlx::query_as::<_, Run>("SELECT * FROM run WHERE id = ?")
        .bind(run_id)
        .fetch_one(pool)
        .await else { return };

    let files: Vec<&str> = run.get_files
        .split(',')
        .map(|f| f.trim())
        .filter(|f| !f.is_empty())
        .collect();

    let Some(first) = files.first() else { return };
    let local_file = format!("{}/{}", run.local_directory.trim_end_matches('/'), first);
    let records = parse(&local_file, "default_parser.toml");

    let Ok(json) = serde_json::to_string(&records) else { return };

    let _ = sqlx::query("UPDATE run SET records_json = ? WHERE id = ?")
        .bind(json)
        .bind(run_id)
        .execute(pool)
        .await;
}

// #[server]
// pub async fn parse_logs(run_id: i64) -> Result<Vec<Record>, ServerFnError> {
//     use crate::app_state::AppState;
//     use crate::log_parser::parse;
//     let app_state = use_context::<AppState>().ok_or(ServerFnError::new("expected context"))?;
//     let pool = app_state.pool();
//     let get_files: String = sqlx::query_as("SELECT get_files FROM run WHERE id = ?")
//         .bind(run_id)
//         .fetch_one(pool)
//         .await?;
//
//     let files: Vec<&str> = get_files
//         .split(',')
//         .map(|f| f.trim())
//         .filter(|f| !f.is_empty())
//         .collect();
//     let local_file = format!("{}/{}", run.local_directory.trim_end_matches('/'), files[0]);
//     Ok(parse(&local_file, "default_parser.toml"))
// }

#[server]
pub async fn get_run_records(run_id: i64) -> Result<Vec<Record>, ServerFnError> {
    use crate::app_state::AppState;
    let app_state = use_context::<AppState>().ok_or(ServerFnError::new("expected context"))?;
    let pool = app_state.pool();
    
    let run: Run = sqlx::query_as(
        "SELECT * FROM run WHERE id = ?"
    )
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
        records.chunks(chunk_size)
            .map(|chunk| {
                let mut avg = Record::new();
                for key in chunk[0].keys() {
                    let mean = chunk.iter()
                        .filter_map(|r| r.get(key))
                        .sum::<f64>() / chunk.len() as f64;
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

#[component]
pub fn RunList(project_id: i64) -> impl IntoView {
    let runs_resource = LocalResource::new(move || async move {
        get_runs(project_id).await
    });
    provide_context(runs_resource);
    let servers_resource = LocalResource::new(|| async { get_servers().await });

    let add_modal_opened = RwSignal::new(false);

    view! {
        <div class="flex flex-col bg-slate-100 p-3 gap-1">
            <span class="text-center">"Runs"</span>
            <Transition
                fallback=move || view! { <span>"Loading"</span> }
            >
                    {move || match runs_resource.get() {
                        Some(Ok(runs)) => runs.into_iter().map(|run| {
                            let run_clone = run.clone();
                            view! {
                                <div class="flex items-center items-stretch">
                                    <A href="" attr:class="flex grow rounded hover:bg-slate-200 items-center pr-3">
                                        <RunStatus run_id=run.id/>
                                        <span class="align-middle">{run_clone.name}</span>
                                    </A>
                                    <RunModifyButton run=run/>
                                </div>
                            }
                        }).collect_view().into_any(),
                        _ => view! { <span>"Not found"</span> }.into_any(),
                    }}
            </Transition>
            <button
                class="px-3 py-1 rounded bg-slate-200 text-black hover:bg-slate-300"
                on:click=move |_| {
                    add_modal_opened.set(true);
                }
            >
                <div class="flex items-center gap-3 align-middle">
                    <Plus/>
                    <span>"Add run"</span>
                </div>
            </button>
        </div>
        {move || add_modal_opened.get().then(|| view!{
            <RunAddModal add_modal_opened=add_modal_opened project_id=project_id servers_resource=servers_resource/>
        })}
    }.into_any()
}



#[component]
fn RunModifyButton(
    run: Run,
) -> impl IntoView {
    let dropdown_opened = RwSignal::new(false);
    let edit_modal_opened = RwSignal::new(false);
    let delete_modal_opened = RwSignal::new(false);
    let run_clone = run.clone();

    view! {
        <div class="relative">
            <button class="hover:bg-slate-200 rounded p-1 align-middle"
                on:click=move |_| dropdown_opened.update(|v| *v=!*v)
            >
                <Ellipsis/>
            </button>
            <Menu opened=dropdown_opened>
                <button
                    class="px-3 py-1 hover:bg-gray-100 text-left"
                    on:click=move |_| {
                        dropdown_opened.set(false);
                    }
                >
                    "View logs"
                </button>
                <button
                    class="px-3 py-1 hover:bg-gray-100 text-left"
                    on:click=move |_| {
                        edit_modal_opened.set(true);
                        dropdown_opened.set(false);
                    }
                >
                    "Edit"
                </button>
                <button
                    class="px-3 py-1 hover:bg-red-100 text-red-500 text-left"
                    on:click=move |_| {
                        delete_modal_opened.set(true);
                        dropdown_opened.set(false);
                    }
                >
                    "Delete"
                </button>
            </Menu>
        </div>
        <RunEditModal run=run_clone edit_modal_opened=edit_modal_opened/>
        <RunDeleteModal run=run delete_modal_opened=delete_modal_opened/>
    }
}


#[component]
fn RunDeleteModal(
    run: Run,
    delete_modal_opened: RwSignal<bool>,
) -> impl IntoView {
    let run_name = StoredValue::new(run.name);
    let run_id = run.id;
    let delete_action = Action::new(move |_: &()| async move {
        let _ = delete_run(run_id).await;
    });
    Effect::new(move |_| {
        if let Some(_) = delete_action.value().get() {
            let runs_resource = expect_context::<LocalResource<Result<Vec<Run>, ServerFnError>>>();
            runs_resource.refetch();
            delete_modal_opened.set(false);
        }
    });
    view! {
        <Modal opened=delete_modal_opened>
            <div class="flex flex-col gap-4 p-5 max-w-sm">
                <div class="text-center">"Delete run"</div>
                <p class="text-center">
                    "Are you sure you want to delete "{run_name.get_value()}
                    "?"
                </p>
                <div class="flex justify-end gap-2">
                    <button
                        autofocus
                        class="px-3 py-1 bg-gray-100 rounded hover:bg-gray-200"
                        on:click=move |_| delete_modal_opened.set(false)
                    >
                        "Cancel"
                    </button>
                    <button
                        class="px-3 py-1 bg-red-500 text-white hover:bg-red-600 rounded"
                        disabled=delete_action.pending()
                        on:click=move |_| { delete_action.dispatch(()); }
                    >
                        "Delete"
                    </button>
                </div>
            </div>
        </Modal>
    }
}

#[component]
fn RunAddModal(
    add_modal_opened: RwSignal<bool>,
    project_id: i64,
    servers_resource: LocalResource<Result<Vec<Server>, ServerFnError>>,
) -> impl IntoView {
    let submit_action = ServerAction::<CreateRun>::new();
    Effect::new(move |_| {
        if let Some(_) = submit_action.value().get() {
            let runs_resource = expect_context::<LocalResource<Result<Vec<Run>, ServerFnError>>>();
            runs_resource.refetch();
            add_modal_opened.set(false);
        }
    });
    view! {
        <Modal opened=add_modal_opened>
            <ActionForm action=submit_action>
                <div class="flex flex-col gap-1 p-3">
                    <div class="text-center">"Add project"</div>
                    <input type="hidden" name="project_id" value=project_id/>
                    <label class="flex items-center justify-between gap-3">
                        "Server"
                        <ServerSelect servers_resource=servers_resource/>
                    </label>
                    <div class="flex justify-end gap-2">
                        <button
                            autofocus
                            class="px-3 py-1 bg-gray-100 rounded hover:bg-gray-200"
                            on:click=move |e| {
                                e.prevent_default();
                                add_modal_opened.set(false);
                            }
                        >
                            "Cancel"
                        </button>
                        <button
                            class="px-3 py-1 bg-blue-500 text-white hover:bg-blue-600 rounded"
                            type="submit"
                            disabled=submit_action.pending()
                        >
                            "Add"
                        </button>
                    </div>
                </div>
            </ActionForm>
        </Modal>
    }
}

#[component]
fn ServerSelect(
    servers_resource: LocalResource<Result<Vec<Server>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <Transition fallback=move || view! {
            <select class="border rounded px-1 py-1" disabled>
                <option>"Loading"</option>
            </select>
        }>
            {move || match servers_resource.get() {
                Some(Ok(servers)) if !servers.is_empty() => view! {
                    <select class="border rounded px-1 py-1" name="server_id">
                        {servers.into_iter().map(|s| view! {
                            <option value=s.id>
                                {format!("{} ({})", s.name, s.address)}
                            </option>
                        }).collect_view()}
                    </select>
                }.into_any(),
                Some(Ok(_)) => view! {
                    <select class="border rounded px-1 py-1" disabled>
                        <option>"No servers configured"</option>
                    </select>
                }.into_any(),
                _ => view! {
                    <select class="border rounded px-1 py-1" disabled>
                        <option>"Failed to load servers"</option>
                    </select>
                }.into_any(),
            }}
        </Transition>
    }
}

#[component]
fn RunEditModal(
    run: Run,
    edit_modal_opened: RwSignal<bool>,
) -> impl IntoView {
    let run = StoredValue::new(run);
    let submit_action = ServerAction::<UpdateRun>::new();
    Effect::new(move |_| {
        if let Some(_) = submit_action.value().get() {
            let runs_resource = expect_context::<LocalResource<Result<Vec<Run>, ServerFnError>>>();
            runs_resource.refetch();
            edit_modal_opened.set(false);
        }
    });
    view! {
        <Modal opened=edit_modal_opened>
            <ActionForm action=submit_action>
                <div class="flex flex-col gap-1 p-3">
                    <div class="text-center">"Edit run"</div>
                    <input type="hidden" name="run[id]" value=run.read_value().id/>
                    <input type="hidden" name="run[project_id]" value=run.read_value().project_id/>
                    <input type="hidden" name="run[server_id]" value=run.read_value().server_id/>
                    <RunFormFields
                        name=run.read_value().name.clone()
                        local_directory=run.read_value().local_directory.clone()
                        remote_directory=run.read_value().remote_directory.clone()
                        post_files=run.read_value().post_files.clone()
                        get_files=run.read_value().get_files.clone()
                        config_json=run.read_value().config_json.clone()
                        notes=run.read_value().notes.clone()
                    />
                    <div class="flex justify-end gap-2">
                        <button
                            autofocus
                            class="px-3 py-1 bg-gray-100 rounded hover:bg-gray-200"
                            on:click=move |e| {
                                e.prevent_default();
                                edit_modal_opened.set(false);
                            }
                        >
                            "Cancel"
                        </button>
                        <button
                            class="px-3 py-1 bg-blue-500 text-white hover:bg-blue-600 rounded"
                            type="submit"
                            disabled=submit_action.pending()
                        >
                            "Save"
                        </button>
                    </div>
                </div>
            </ActionForm>
        </Modal>
    }
}


#[component]
fn RunStatus(run_id: i64) -> impl IntoView {
    // Only render on client - starts as None on SSR
    let refresh = RwSignal::new(0u32);
    // let mounted = RwSignal::new(false);
    // Effect::new(move |_| mounted.set(true));

    let alive = LocalResource::new(move || {
        let _ = refresh.get();
        async move {
            let result = download(run_id).await.unwrap_or(false);
            // let result = true;
            set_timeout(
                move || refresh.update(|n| *n += 1),
                std::time::Duration::from_millis(10_000),
            );
            result
        }
    });

    move || {
        // if !mounted.get() {
        //     return view! { <Dot stroke_width=8 color="var(--color-yellow-500)"/> }.into_any();
        // }
        view! {
            <div class="align-middle">
                <Transition fallback=move || view! {
                    <Dot stroke_width=8 color="var(--color-yellow-500)"/>
                }>
                    <Dot
                        stroke_width=8
                        color=move || {
                            if alive.get().unwrap_or(false) {
                                "var(--color-green-500)"
                            } else {
                                "var(--color-red-500)"
                            }
                        }
                    />
                </Transition>
            </div>
        }
        .into_any()
    }
}



#[component]
fn RunFormFields(
    name: String,
    local_directory: String,
    remote_directory: String,
    post_files: String,
    get_files: String,
    config_json: String,
    notes: String,
) -> impl IntoView {
    view! {
        <label class="flex items-center justify-between gap-3">
            "Run name"
            <input class="border rounded px-1 py-1" value=name name="run[name]"/>
        </label>
        <label class="flex items-center justify-between gap-3">
            "Local directory"
            <input class="border rounded px-1 py-1" value=local_directory name="run[local_directory]"/>
        </label>
        <label class="flex items-center justify-between gap-3">
            "Remote directory"
            <input class="border rounded px-1 py-1" value=remote_directory name="run[remote_directory]"/>
        </label>
        <label class="flex items-center justify-between gap-3">
            "Source files"
            <input class="border rounded px-1 py-1" value=post_files name="run[post_files]"/>
        </label>
        <label class="flex items-center justify-between gap-3">
            "Output files"
            <input class="border rounded px-1 py-1" value=get_files name="run[get_files]"/>
        </label>
        <label class="flex items-center justify-between gap-3">
            "Notes"
            <input class="border rounded px-1 py-1" value=notes name="run[notes]"/>
        </label>
        <label class="flex items-center justify-between gap-3">
            "Config"
            <input class="border rounded px-1 py-1" value=config_json name="run[config_json]"/>
        </label>
    }
}
