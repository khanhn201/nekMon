use leptos::prelude::*;
use leptos_router::components::A;

use lucide_leptos::{Ellipsis, Plus};

use crate::components::modal::{Menu, Modal};
use crate::model::Project;

/// ------------------------------
/// Server functions
/// ------------------------------

#[server]
pub async fn get_projects() -> Result<Vec<Project>, ServerFnError> {
    use crate::app_state::AppState;
    let app_state = use_context::<AppState>().expect("could not find AppState in context");
    let pool = app_state.pool();
    let projects: Vec<Project> = sqlx::query_as("SELECT * FROM project")
        .fetch_all(pool)
        .await?;
    Ok(projects)
}

#[server]
pub async fn create_project(project: Project) -> Result<(), ServerFnError> {
    use crate::app_state::AppState;
    let app_state = use_context::<AppState>().expect("could not find AppState in context");
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
    let app_state = use_context::<AppState>().expect("could not find AppState in context");
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
    let app_state = use_context::<AppState>().expect("could not find AppState in context");
    let pool = app_state.pool();
    sqlx::query("DELETE FROM project WHERE id = ?")
        .bind(project_id)
        .execute(pool)
        .await?;
    Ok(())
}


/// ------------------------------
/// Components
/// ------------------------------

#[component]
pub fn ProjectList() -> impl IntoView {
    let projects_resource = Resource::new(|| {}, |_| async { get_projects().await });

    let add_modal_opened = RwSignal::new(false);

    view! {
        <div class="flex flex-col bg-slate-100 p-3 gap-1">
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
                                    <div class="flex items-center items-stretch">
                                        <A href=href attr:class="flex grow rounded hover:bg-slate-200 items-center">
                                            <span class="ml-[24px] align-middle">{project_clone.name.clone()}</span>
                                        </A>
                                        <ProjectModifyButton
                                            project=project
                                            projects_resource=projects_resource
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
            <ProjectAddModal add_modal_opened=add_modal_opened projects_resource=projects_resource/>
        })}
    }
}


#[component]
fn ProjectModifyButton(
    project: Project,
    projects_resource: Resource<Result<Vec<Project>, ServerFnError>>,
) -> impl IntoView {
    let dropdown_opened = RwSignal::new(false);
    let edit_modal_opened = RwSignal::new(false);
    let delete_modal_opened = RwSignal::new(false);
    let project_clone = project.clone();

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
        <ProjectEditModal project=project_clone edit_modal_opened=edit_modal_opened projects_resource=projects_resource/>
        <ProjectDeleteModal project=project delete_modal_opened=delete_modal_opened projects_resource=projects_resource/>
    }
}



#[component]
fn ProjectAddModal(
    add_modal_opened: RwSignal<bool>,
    projects_resource: Resource<Result<Vec<Project>, ServerFnError>>,
) -> impl IntoView {
    let submit_action = ServerAction::<CreateProject>::new();
    Effect::new(move |_| {
        if let Some(_) = submit_action.value().get() {
            add_modal_opened.set(false);
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
fn ProjectEditModal(
    project: Project,
    edit_modal_opened: RwSignal<bool>,
    projects_resource: Resource<Result<Vec<Project>, ServerFnError>>,
) -> impl IntoView {
    let project = StoredValue::new(project);
    let submit_action = ServerAction::<UpdateProject>::new();
    Effect::new(move |_| {
        if let Some(_) = submit_action.value().get() {
            edit_modal_opened.set(false);
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
fn ProjectFormFields(
    #[prop(default = String::new())] name: String,
    #[prop(default = String::new())] local_directory: String,
    #[prop(default = String::new())] src_directory: String,
    #[prop(default = String::new())] post_files: String,
    #[prop(default = String::new())] get_files: String,
) -> impl IntoView {
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



#[component]
fn ProjectDeleteModal(
    project: Project,
    delete_modal_opened: RwSignal<bool>,
    projects_resource: Resource<Result<Vec<Project>, ServerFnError>>,
) -> impl IntoView {
    let project_name = StoredValue::new(project.name);
    let project_id = project.id;
    let delete_action = Action::new(move |_: &()| async move {
        let _ = delete_project(project_id).await;
        projects_resource.refetch();
    });
    Effect::new(move |_| {
        if let Some(()) = delete_action.value().get() {
            delete_modal_opened.set(false);
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
                        on:click=move |_| { delete_action.dispatch(()); }
                    >
                        "Delete"
                    </button>
                </div>
            </div>
        </Modal>
    }
}
