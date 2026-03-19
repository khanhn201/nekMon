use leptos::prelude::*;
use leptos_router::components::A;

use lucide_leptos::{Ellipsis, Plus};

use crate::components::modal::{Menu, Modal};
use crate::models::project::*;

/// ------------------------------
/// Components
/// ------------------------------

#[component]
pub fn ProjectList() -> impl IntoView {
    let projects_resource = LocalResource::new(|| async { get_projects().await });
    provide_context(projects_resource);

    let add_modal_opened = RwSignal::new(false);

    view! {
        <div class="flex grow flex-col bg-slate-100 p-3 gap-1">
            <span class="text-center">"Projects"</span>

            <Transition
                fallback=move || view! { <span>"Loading"</span> }
            >
                { move || {
                    match projects_resource.get() {

                        Some(Ok(projects)) =>
                            projects.into_iter().map(|project| {
                                let href = format!("/project/{}", project.id);
                                let project_clone = project.clone();
                                view! {
                                    <div class="flex items-center items-stretch rounded hover:bg-slate-200">
                                        <A href=href attr:class="flex grow items-center pr-3">
                                            <span class="ml-[33px] align-middle">{project_clone.name.clone()}</span>
                                        </A>
                                        <ProjectModifyButton
                                            project=project
                                        />
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
                    <span>"Add project"</span>
                </div>
            </button>
        </div>
        {move || add_modal_opened.get().then(|| view!{
            <ProjectAddModal add_modal_opened=add_modal_opened/>
        })}
    }
}

#[component]
fn ProjectModifyButton(project: Project) -> impl IntoView {
    let dropdown_opened = RwSignal::new(false);
    let edit_modal_opened = RwSignal::new(false);
    let delete_modal_opened = RwSignal::new(false);
    let project_clone = project.clone();

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
        <ProjectEditModal project=project_clone edit_modal_opened=edit_modal_opened/>
        <ProjectDeleteModal project=project delete_modal_opened=delete_modal_opened/>
    }
}

#[component]
fn ProjectAddModal(add_modal_opened: RwSignal<bool>) -> impl IntoView {
    let submit_action = ServerAction::<CreateProject>::new();
    Effect::new(move |_| {
        if let Some(_) = submit_action.value().get() {
            add_modal_opened.set(false);
            let projects_resource =
                expect_context::<LocalResource<Result<Vec<Project>, ServerFnError>>>();
            projects_resource.refetch();
        }
    });
    view! {
        <Modal opened=add_modal_opened>
            <ActionForm action=submit_action>
                <div class="flex flex-col gap-1 p-3">
                    <div class="text-center">"Add project"</div>
                    <ProjectFormFields/>
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
fn ProjectEditModal(project: Project, edit_modal_opened: RwSignal<bool>) -> impl IntoView {
    let project = StoredValue::new(project);
    let submit_action = ServerAction::<UpdateProject>::new();
    Effect::new(move |_| {
        if let Some(_) = submit_action.value().get() {
            edit_modal_opened.set(false);
            let projects_resource =
                expect_context::<LocalResource<Result<Vec<Project>, ServerFnError>>>();
            projects_resource.refetch();
        }
    });
    view! {
        <Modal opened=edit_modal_opened>
            <ActionForm action=submit_action>
                <div class="flex flex-col gap-1 p-3">
                    <div class="text-center">"Edit project"</div>
                    <input type="hidden" name="project[id]" value=project.read_value().id/>
                    <ProjectFormFields
                        name=project.read_value().name.clone()
                        local_directory=project.read_value().local_directory.clone()
                        src_directory=project.read_value().src_directory.clone()
                        post_files=project.read_value().post_files.clone()
                        get_files=project.read_value().get_files.clone()
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
fn ProjectDeleteModal(project: Project, delete_modal_opened: RwSignal<bool>) -> impl IntoView {
    let project_name = StoredValue::new(project.name);
    let delete_action = ServerAction::<DeleteProject>::new();
    Effect::new(move |_| {
        if let Some(_) = delete_action.value().get() {
            delete_modal_opened.set(false);
            let projects_resource =
                expect_context::<LocalResource<Result<Vec<Project>, ServerFnError>>>();
            projects_resource.refetch();
        }
    });
    view! {
        <Modal opened=delete_modal_opened>
            <div class="flex flex-col gap-4 p-5 max-w-sm">
                <div class="text-center">"Delete project"</div>
                <p class="text-center">
                    "Are you sure you want to delete "{project_name.get_value()}
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
                        on:click=move |_| { delete_action.dispatch(DeleteProject { project_id: project.id } ); }
                    >
                        "Delete"
                    </button>
                </div>
            </div>
        </Modal>
    }
}

#[component]
fn ProjectFormFields(
    #[prop(default = String::new())] name: String,
    #[prop(default = String::new())] local_directory: String,
    #[prop(default = String::new())] src_directory: String,
    #[prop(default = String::new())] post_files: String,
    #[prop(default = String::new())] get_files: String,
) -> impl IntoView {
    // TODO: getfiles and postfiles as list/table instead of having user input comma separated
    // TODO: let user choose the logfile
    // TODO: Scroll end of textbox
    view! {
        <label class="flex items-center justify-between gap-3">
            "Project name"
            <input class="border rounded px-1 py-1" value=name name="project[name]"/>
        </label>
        <label class="flex items-center justify-between gap-3">
            "Local directory"
            <input class="border rounded px-1 py-1" value=local_directory name="project[local_directory]"/>
        </label>
        <label class="flex items-center justify-between gap-3">
            "Source directory"
            <input class="border rounded px-1 py-1" value=src_directory name="project[src_directory]"/>
        </label>
        <label class="flex items-center justify-between gap-3">
            "Source files"
            <input class="border rounded px-1 py-1" value=post_files name="project[post_files]"/>
        </label>
        <label class="flex items-center justify-between gap-3">
            "Output files"
            <input class="border rounded px-1 py-1" value=get_files name="project[get_files]"/>
        </label>
    }
}
