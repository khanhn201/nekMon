use leptos::prelude::*;

use leptos_router::{
    hooks::{use_params},
    params::Params,
};

use crate::model::{ Project };
use crate::components::run_list::RunList;

#[derive(Params, PartialEq, Clone, Debug)]
pub struct ProjectParams {
    pub id: i64,
}



#[server]
pub async fn get_project(project_id: i64) -> Result<Project, ServerFnError> {
    use crate::app_state::AppState;
    let app_state: AppState =
        use_context::<AppState>().ok_or(ServerFnError::new("expected context"))?;
    let pool = app_state.pool();
    let project: Project = sqlx::query_as("SELECT * FROM project WHERE id = ?")
        .bind(project_id)
        .fetch_one(pool)
        .await?;
    Ok(project)
}


#[component]
pub fn ProjectView() -> impl IntoView {
    move || {
        let params = use_params::<ProjectParams>();
        let id = params.get().unwrap_or(ProjectParams{id:0}).id;
        let project_resource = LocalResource::new(move || async move {
            get_project(id).await
        });
        view! {
            <div class="flex flex-col bg-slate-100 gap-1 grow">
                <Transition
                    fallback=move || view! { <span>"Loading"</span> }
                >
                    { move || {
                        match project_resource.get() {
                            Some(Ok(project)) => view!{
                                    <span class="text-center p-3">"Project "{project.name}</span>
                                    <div class="flex flex-row grow bg-slate-100 gap-1">
                                        <RunList project_id=id/>
                                        <div class="flex grow">Graphs</div>
                                    </div>
                                }.into_any(),
                            _ => view!{ <span>"Not found"</span> }.into_any()
                        }
                    }}
                </Transition>
            </div>
        }.into_any()
    }
}
