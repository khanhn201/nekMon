use leptos::prelude::*;

use leptos_router::{hooks::use_params, params::Params};

use crate::components::chart::Chart;
use crate::components::run_list::RunList;
use crate::models::project::*;
use crate::models::run::*;

use std::collections::HashMap;
use std::collections::HashSet;


pub const RUN_COLORS: &[&str] = &[
    "#4c78a8", "#f58518", "#e45756", "#72b7b2",
    "#54a24b", "#eeca3b", "#b279a2", "#ff9da6",
    "#9d755d", "#bab0ac",
]; // tableau10

#[derive(Params, PartialEq, Clone, Debug)]
pub struct ProjectParams {
    pub id: i64,
}

#[component]
pub fn ProjectView() -> impl IntoView {
    move || {
        let params = use_params::<ProjectParams>();
        let id = params.get().unwrap_or(ProjectParams { id: 0 }).id;
        let project_resource = LocalResource::new(move || async move { get_project(id).await });
        let runs_resource = LocalResource::new(move || async move { get_runs(id).await });
        let hovered_id: RwSignal<Option<i64>> = RwSignal::new(None);
        let hidden_ids: RwSignal<HashSet<i64>> = RwSignal::new(HashSet::new());
        view! {
            <div class="flex flex-col bg-slate-100 gap-1 size-full min-w-0 min-h-0">
                <Transition
                    fallback=move || view! { <span>"Loading"</span> }
                >
                    { move || {
                        match project_resource.get() {
                            Some(Ok(project)) => view!{
                                    <span class="text-center p-3">"Project "{project.name}</span>
                                }.into_any(),
                            _ => view!{ <span>"Not found"</span> }.into_any()
                        }
                    }}
                </Transition>
                <div class="flex size-full flex-row bg-slate-100 gap-1 min-w-0 min-h-0">
                    <Transition
                        fallback=move || view! { <span>"Loading"</span> }
                    >
                        {move || match runs_resource.get() {
                            Some(Ok(runs)) => {
                                let run_ids: Vec<i64> = runs.iter()
                                    .map(|f| { f.id })
                                    .collect();
                                let colors: HashMap<i64, &str> = run_ids.iter()
                                    .enumerate()
                                    .map(|(i, &id)| (id, RUN_COLORS[i % RUN_COLORS.len()]))
                                    .collect();

                                view!{
                                    <RunList
                                        project_id=id
                                        runs_resource=runs_resource
                                        colors=colors.clone()
                                        hovered_id=hovered_id
                                        hidden_ids=hidden_ids
                                    />
                                    <Chart
                                        run_ids=run_ids 
                                        colors=colors
                                        hovered_id=hovered_id
                                        hidden_ids=hidden_ids
                                    />
                                }.into_any()
                            },
                            _ => view!{ <span>"Not found"</span> }.into_any()
                        }}
                    </Transition>
                </div>
            </div>
        }
        .into_any()
    }
}
