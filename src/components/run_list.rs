use leptos::prelude::*;
use leptos_router::components::A;

use lucide_leptos::{Dot, Ellipsis, Plus, Eye, EyeOff};

use crate::components::modal::{Menu, Modal};
use crate::models::run::*;
use crate::models::server::*;

use std::collections::HashMap;
use std::collections::HashSet;

#[component]
pub fn RunList(
    project_id: i64,
    runs_resource: LocalResource<Result<Vec<Run>, ServerFnError>>,
    colors: HashMap<i64, &'static str>,
    hovered_id: RwSignal<Option<i64>>,
    hidden_ids: RwSignal<HashSet<i64>>,
) -> impl IntoView {
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
                            let run_id = run.id;
                            let color = colors.get(&run_id).copied().unwrap_or("#94a3b8");
                            view! {
                                <div 
                                    class="flex items-center items-stretch hover:bg-slate-200 rounded"
                                    on:mouseenter=move |_| hovered_id.set(Some(run_id))
                                    on:mouseleave=move |_| hovered_id.set(None)
                                >
                                    <button
                                        class="hover:bg-slate-300 rounded p-1 align-middle"
                                        on:click=move |_| hidden_ids.update(|set| {
                                            if set.contains(&run_id) { set.remove(&run_id); } else { set.insert(run_id); }
                                        })
                                    >
                                        {move || if hidden_ids.get().contains(&run_id) {
                                            view! { <EyeOff color="var(--color-neutral-500)"/> }.into_any()
                                        } else {
                                            view! { <Eye color="var(--color-neutral-500)"/> }.into_any()
                                        }}
                                    </button>
                                    <button class="hover:bg-slate-300 rounded p-1 align-middle">
                                        <Dot
                                            stroke_width=12
                                            color=move || color
                                        />
                                    </button>
                                    <A href="" attr:class="flex grow items-center pr-3">
                                        <RunPing run_id=run.id/>
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
fn RunModifyButton(run: Run) -> impl IntoView {
    let dropdown_opened = RwSignal::new(false);
    let edit_modal_opened = RwSignal::new(false);
    let delete_modal_opened = RwSignal::new(false);
    let run_clone = run.clone();
    let run_id = run.id;
    let run_status = RwSignal::new(run.status.clone());

    let toggle_status_action = ServerAction::<SetRunStatus>::new();
    let upload_action = ServerAction::<Upload>::new();

    Effect::new(move |_| {
        if let Some(_) = toggle_status_action.value().get() {
            let runs_resource = expect_context::<LocalResource<Result<Vec<Run>, ServerFnError>>>();
            runs_resource.refetch();
        }
    });

    view! {
        <div class="relative">
            <button class="hover:bg-slate-300 rounded p-1 align-middle"
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
                    class=move || format!(
                        "px-3 py-1 text-left {}",
                        if run_status.get() == RunStatus::Running {
                            "hover:bg-yellow-100 text-yellow-500"
                        } else {
                            "hover:bg-green-100 text-green-500"
                        }
                    )
                    disabled=toggle_status_action.pending()
                    on:click=move |_| {
                        let next = if run_status.get() == RunStatus::Running {
                            RunStatus::Completed
                        } else {
                            RunStatus::Running
                        };
                        toggle_status_action.dispatch(SetRunStatus {run_id: run_id, status: next});
                        dropdown_opened.set(false);
                    }
                >
                    {move || if run_status.get() == RunStatus::Running {
                        "Mark completed"
                    } else {
                        "Mark running"
                    }}
                </button>
                <button
                    class="px-3 py-1 hover:bg-gray-100 text-left"
                    on:click=move |_| {
                        upload_action.dispatch(Upload { run_id: run_id} );
                        dropdown_opened.set(false);
                    }
                >
                    "Upload source files"
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
fn RunDeleteModal(run: Run, delete_modal_opened: RwSignal<bool>) -> impl IntoView {
    let run_name = StoredValue::new(run.name);
    let run_id = run.id;
    let delete_action = ServerAction::<DeleteRun>::new();
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
                        on:click=move |_| { delete_action.dispatch(DeleteRun { run_id: run_id } ); }
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
fn RunEditModal(run: Run, edit_modal_opened: RwSignal<bool>) -> impl IntoView {
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
                        src_directory=run.read_value().src_directory.clone()
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
fn RunPing(run_id: i64) -> impl IntoView {
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
        view! {
            <div class="align-middle p-1">
                <Transition fallback=move || view! {
                    <Dot stroke_width=12 color="var(--color-yellow-500)"/>
                }>
                    <Dot
                        stroke_width=12
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
    src_directory: String,
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
            "Source directory"
            <input class="border rounded px-1 py-1" value=src_directory name="run[src_directory]"/>
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
