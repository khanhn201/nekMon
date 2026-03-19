use leptos::prelude::*;

use lucide_leptos::{Dot, Ellipsis, Plus};

use crate::components::modal::{Menu, Modal};
use crate::models::server::*;

/// ------------------------------
/// Components
/// ------------------------------

#[component]
pub fn ServerList() -> impl IntoView {
    let servers_resource = LocalResource::new(|| async { get_servers().await });
    provide_context(servers_resource);

    let add_modal_opened = RwSignal::new(false);

    view! {
        <div class="flex flex-col bg-slate-100 p-3 gap-1">
            <span class="text-center">"Servers"</span>

            <Transition
                fallback=move || view! { <span>"Loading"</span> }
            >
                { move || {
                    match servers_resource.get() {
                        Some(Ok(servers)) =>
                            servers.into_iter().map(|server| {
                                let server_clone = server.clone();
                                view! {
                                    <div class="flex items-stretch">
                                        <div class="flex grow items-center">
                                            <ServerPing server=server/>
                                            <span class="grow align-middle">{server_clone.name.clone()}</span>
                                        </div>
                                        <ServerModifyButton server=server_clone/>
                                    </div>
                                }
                            }).collect_view().into_any(),
                        _ => view!{ <span>"Error"</span> }.into_any()
                    }
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
                    <span>"Add server"</span>
                </div>
            </button>
        </div>
        {move || add_modal_opened.get().then(|| view!{
            <ServerAddModal add_modal_opened=add_modal_opened/>
        })}
    }
}

#[component]
fn ServerPing(server: Server) -> impl IntoView {
    // Only render on client - starts as None on SSR
    let refresh = RwSignal::new(0u32);
    // let mounted = RwSignal::new(false);
    // Effect::new(move |_| mounted.set(true));

    let alive = LocalResource::new(move || {
        let _ = refresh.get();
        let server = server.clone();
        async move {
            let result = get_alive_status(server).await.unwrap_or(false);
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
fn ServerModifyButton(server: Server) -> impl IntoView {
    let dropdown_opened = RwSignal::new(false);
    let edit_modal_opened = RwSignal::new(false);
    let delete_modal_opened = RwSignal::new(false);
    let server_clone = server.clone();

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
        <ServerEditModal server=server edit_modal_opened=edit_modal_opened/>
        <ServerDeleteModal server=server_clone delete_modal_opened=delete_modal_opened/>
    }
}

#[component]
fn ServerAddModal(add_modal_opened: RwSignal<bool>) -> impl IntoView {
    let submit_action = ServerAction::<CreateServer>::new();
    Effect::new(move |_| {
        if let Some(_) = submit_action.value().get() {
            add_modal_opened.set(false);
            let servers_resource =
                expect_context::<LocalResource<Result<Vec<Server>, ServerFnError>>>();
            servers_resource.refetch();
        }
    });
    view! {
        <Modal opened=add_modal_opened>
            <ActionForm action=submit_action>
                <div class="flex flex-col gap-1 p-3">
                    <div class="text-center">"Add server"</div>
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
fn ServerEditModal(server: Server, edit_modal_opened: RwSignal<bool>) -> impl IntoView {
    let server = StoredValue::new(server);
    let submit_action = ServerAction::<UpdateServer>::new();
    Effect::new(move |_| {
        if let Some(_) = submit_action.value().get() {
            edit_modal_opened.set(false);
            let servers_resource =
                expect_context::<LocalResource<Result<Vec<Server>, ServerFnError>>>();
            servers_resource.refetch();
        }
    });
    view! {
        <Modal opened=edit_modal_opened>
            <ActionForm action=submit_action>
                <div class="flex flex-col gap-1 p-3">
                    <div class="text-center">"Edit server"</div>
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
fn ServerDeleteModal(server: Server, delete_modal_opened: RwSignal<bool>) -> impl IntoView {
    let server_name = StoredValue::new(server.name);
    let delete_action = ServerAction::<DeleteServer>::new();
    Effect::new(move |_| {
        if let Some(_) = delete_action.value().get() {
            delete_modal_opened.set(false);
            let servers_resource =
                expect_context::<LocalResource<Result<Vec<Server>, ServerFnError>>>();
            servers_resource.refetch();
        }
    });
    view! {
        <Modal opened=delete_modal_opened>
            <div class="flex flex-col gap-4 p-5 max-w-sm">
                <div class="text-center">"Delete server"</div>
                <p class="text-center">
                    "Are you sure you want to delete "{server_name.get_value()}
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
                        on:click=move |_| { delete_action.dispatch(DeleteServer {server_id: server.id}); }
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
