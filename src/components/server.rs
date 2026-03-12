use leptos::prelude::*;

use leptos_use::{use_interval, UseIntervalReturn};


use lucide_leptos::{Dot, Ellipsis, Plus};

use crate::model::Server;
use crate::components::modal::{Modal, Menu};



/// ------------------------------
/// Server functions
/// ------------------------------

#[server]
pub async fn get_servers() -> Result<Vec<Server>, ServerFnError> {
    use crate::app_state::AppState;

    let app_state = use_context::<AppState>().expect("could not find AppState in context");

    let pool = app_state.pool();
    let servers: Vec<Server> = sqlx::query_as(
        "SELECT * FROM server"
    )
    .fetch_all(pool)
    .await?;
    Ok(servers)
}

#[server]
pub async fn get_alive_status(server: Server) -> Result<bool, ServerFnError> {
    use crate::app_state::AppState;
    
    let app_state = use_context::<AppState>().expect("AppState missing");
    Ok(app_state.get_ssh_client(&server).await.is_some())
}

#[server]
pub async fn create_server(
    server: Server
) -> Result<(), ServerFnError> {
    use crate::app_state::AppState;
    let app_state = use_context::<AppState>().expect("could not find AppState in context");
    let pool = app_state.pool();
    let _server = sqlx::query_as::<_, Server>(
        r#"INSERT INTO server (name, address, username, remote_directory, key_file_path, port)
         VALUES (?, ?, ?, ?, ?, ?)
         RETURNING *"#
    )
    .bind(server.name).bind(server.address).bind(server.username).bind(server.remote_directory)
    .bind(server.key_file_path).bind(server.port)
    .fetch_one(pool)
    .await?;

    Ok(())
}

#[server]
pub async fn update_server(
    server: Server
) -> Result<(), ServerFnError> {
    use crate::app_state::AppState;
    let app_state = use_context::<AppState>().expect("could not find AppState in context");
    let pool = app_state.pool();
    let _server = sqlx::query_as::<_, Server>(
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
    .bind(server.name).bind(server.address).bind(server.username).bind(server.remote_directory)
    .bind(server.key_file_path).bind(server.port).bind(server.id)
    .fetch_one(pool)
    .await?;

    Ok(())
}


#[server]
pub async fn delete_server(server_id: i64) -> Result<(), ServerFnError> {
    use crate::app_state::AppState;
    let app_state = use_context::<AppState>().expect("could not find AppState in context");
    let pool = app_state.pool();
    sqlx::query("DELETE FROM server WHERE id = ?")
        .bind(server_id)
        .execute(pool)
        .await?;
    Ok(())
}




/// ------------------------------
/// Components
/// ------------------------------



#[component]
pub fn ServerList() -> impl IntoView {
    let servers_resource = Resource::new(
        || {},
        |_| async { get_servers().await }
    );

    let add_modal_opened = RwSignal::new(false);

    view! {
        <div class="flex flex-col bg-slate-100 p-3">
            <span class="px-3 py-1 text-center">
                "Servers"
            </span>

            <Transition
                fallback=move || view! { <span class="px-3 py-1 text-center">"Loading"</span> }
            >
                { move || { 
                    match servers_resource.get() {
                        Some(Ok(servers)) => 
                            servers.into_iter().map(|server| {
                                let server_clone = server.clone();
                                view! {
                                    <div class="flex items-center justify-between">
                                        <div class="flex items-center">
                                            <ServerStatus server=server/>
                                            <span class="py-1 text-left">{server_clone.name.clone()}</span>
                                        </div>
                                        <ServerModifyButton 
                                            server=server_clone
                                            servers_resource=servers_resource
                                        />
                                    </div>
                                }
                            }).collect_view().into_any(),
                        _ => view!{ <span class="text-center">"Error"</span> }.into_any()
                    }
                }}
            </Transition>
        
            <button
                class="px-3 py-1 rounded bg-slate-200 text-black hover:bg-slate-300"
                on:click=move |_| {
                    add_modal_opened.set(true);
                }
            >
                <div class="flex items-center gap-3">
                    <Plus/>
                    <span>"Add Server"</span>
                </div>
            </button>
        </div>
        {move || add_modal_opened.get().then(|| view!{
            <ServerAddModal add_modal_opened=add_modal_opened servers_resource=servers_resource/>
        })}
    }
}

#[component]
fn ServerStatus(server: Server) -> impl IntoView {
    let UseIntervalReturn {
        counter: refresh,
        pause,
        resume,
        ..
    }  = use_interval( 10000 ); // TODO: customize polling rate
    
    // Only render on client - starts as None on SSR
    let (mounted, set_mounted) = signal(false);
    Effect::new(move |_| set_mounted.set(true));

    let alive = LocalResource::new(
        move || {
            refresh.get();
            pause();
            let server = server.clone();
            let resume = resume.clone();
            async move {
                let result = get_alive_status(server).await.unwrap_or(false);
                resume();
                result
            }
        }
    );

    {move || mounted.get().then(|| view! {
        <Transition fallback=move || view! {
            <Dot stroke_width=8 color="var(--color-yellow-500)"/>
        }>
            <Dot
                stroke_width=8
                color=move || if alive.get().unwrap_or(false) {
                    "var(--color-green-500)"
                } else {
                    "var(--color-red-500)"
                }
            />
        </Transition>
    })}
}


#[component]
fn ServerModifyButton(server: Server, servers_resource: Resource<Result<Vec<Server>, ServerFnError>>) -> impl IntoView {
    let dropdown_opened = RwSignal::new(false);
    let edit_modal_opened = RwSignal::new(false);
    let delete_modal_opened = RwSignal::new(false);
    let server_clone = server.clone();

    view! {
        <div class="relative">
            <button class="hover:bg-slate-200 rounded p-1"
                on:click=move |_| dropdown_opened.update(|v| *v=!*v)
            >
                <Ellipsis/>
            </button>
            <Menu opened=dropdown_opened>
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
        <ServerEditModal server=server edit_modal_opened=edit_modal_opened servers_resource=servers_resource/>
        <ServerDeleteModal server=server_clone delete_modal_opened=delete_modal_opened servers_resource=servers_resource/>
    }
}


#[component]
fn ServerAddModal(add_modal_opened: RwSignal<bool>, servers_resource: Resource<Result<Vec<Server>, ServerFnError>>) -> impl IntoView {
    let submit_action = ServerAction::<CreateServer>::new();
    Effect::new(move |_| {
        if let Some(_) = submit_action.input().get() {
            add_modal_opened.set(false);
            servers_resource.refetch();
        }
    });
    view! {
        <Modal opened=add_modal_opened>
            <ActionForm action=submit_action>
                <div class="flex flex-col gap-3 p-3">
                    <div class="text-center">"Add Server"</div>
                    <input type="hidden" name="server[id]" value=0/>
                    <ServerFormFields/>
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
fn ServerEditModal(server: Server, edit_modal_opened: RwSignal<bool>, servers_resource: Resource<Result<Vec<Server>, ServerFnError>>) -> impl IntoView {
    let server = StoredValue::new(server);
    let submit_action = ServerAction::<UpdateServer>::new();
    Effect::new(move |_| {
        if let Some(_) = submit_action.input().get() {
            edit_modal_opened.set(false);
            servers_resource.refetch();
        }
    });
    view! {
        <Modal opened=edit_modal_opened>
            <ActionForm action=submit_action>
                <div class="flex flex-col gap-3 p-3">
                    <div class="text-center">"Edit Server"</div>
                    <input type="hidden" name="server[id]" value=server.read_value().id/>
                    <ServerFormFields
                        name=server.read_value().name.clone()
                        address=server.read_value().address.clone()
                        port=server.read_value().port
                        username=server.read_value().username.clone()
                        remote_directory=server.read_value().remote_directory.clone()
                        key_file_path=server.read_value().key_file_path.clone()
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
fn ServerDeleteModal(
    server: Server,
    delete_modal_opened: RwSignal<bool>,
    servers_resource: Resource<Result<Vec<Server>, ServerFnError>>
) -> impl IntoView {
    let server_name = StoredValue::new(server.name);
    let server_id = server.id;
    let delete_action = Action::new(move |_: &()| {
        async move {
            let _ = delete_server(server_id).await;
            servers_resource.refetch();
        }
    });
    Effect::new(move |_| {
        if let Some(()) = delete_action.value().get() {
            delete_modal_opened.set(false);
        }
    });
    view! {
        <Modal opened=delete_modal_opened>
            <div class="flex flex-col gap-4 p-5 max-w-sm">
                <div class="text-center">"Delete Server"</div>
                <p class="text-center">
                    "Are you sure you want to delete "
                    <span class="font-semibold">{server_name.get_value()}</span>
                    "?"
                </p>
                // TODO: add the number of runs associated
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
fn ServerFormFields(
    #[prop(default = String::new())] name: String,
    #[prop(default = String::new())] address: String,
    #[prop(default = 22u16)] port: u16,
    #[prop(default = String::new())] username: String,
    #[prop(default = String::new())] remote_directory: String,
    #[prop(default = String::new())] key_file_path: String,
) -> impl IntoView {
    view! {
        <label class="flex items-center justify-between gap-3">
            "Server name"
            <input class="border rounded px-1 py-1" value=name name="server[name]"/>
        </label>
        <label class="flex items-center justify-between gap-3">
            "Server address"
            <input class="border rounded px-1 py-1" value=address name="server[address]"/>
        </label>
        <label class="flex items-center justify-between gap-3">
            <span>"Port"</span>
            <input class="border rounded px-1 py-1" type="number" min=1 step=1
                value=port name="server[port]"/>
        </label>
        <label class="flex items-center justify-between gap-3">
            <span>"Username"</span>
            <input class="border rounded px-1 py-1" value=username name="server[username]"/>
        </label>
        <label class="flex items-center justify-between gap-3">
            <span>"Remote directory"</span>
            <input class="border rounded px-1 py-1"
                value=remote_directory name="server[remote_directory]"/>
        </label>
        <label class="flex items-center justify-between gap-3">
            <span>"SSH key file"</span>
            <input class="border rounded px-1 py-1"
                value=key_file_path name="server[key_file_path]"/>
        </label>
    }
}
