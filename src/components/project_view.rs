use leptos::prelude::*;

use leptos_router::{hooks::use_params, params::Params};

use crate::components::chart::Chart;
use crate::components::run_list::RunList;
use crate::models::project::*;
use crate::models::run::*;

use std::collections::HashMap;


pub const RUN_COLORS: &[&str] = &[
    "#4c78a8", "#f58518", "#e45756", "#72b7b2",
    "#54a24b", "#eeca3b", "#b279a2", "#ff9da6",
    "#9d755d", "#bab0ac",
]; // tableau10

#[derive(Params, PartialEq, Clone, Debug)]
pub struct ProjectParams {
    pub id: i64,
}

#[derive(Clone, Debug)]
pub struct RunState {
    pub color: &'static str, 
    pub visible: RwSignal<bool>,
    pub hovered: RwSignal<bool>,
}

#[component]
pub fn ProjectView() -> impl IntoView {
    move || {
        let params = use_params::<ProjectParams>();
        let id = params.get().unwrap_or(ProjectParams { id: 0 }).id;
        let project_resource = LocalResource::new(move || async move { get_project(id).await });
        let runs_resource = LocalResource::new(move || async move { get_runs(id).await });
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
                                let run_states = run_ids
                                    .iter()
                                    .enumerate()
                                    .map(|(i, &id)| {
                                        let color = RUN_COLORS[i % RUN_COLORS.len()];
                                        (id, RunState {
                                            color,
                                            visible: RwSignal::new(true),
                                            hovered: RwSignal::new(false),
                                        })
                                    })
                                    .collect::<HashMap<i64,RunState>>();
                                let run_states = RwSignal::new(run_states);

                                
                                view!{
                                    <RunList project_id=id runs_resource=runs_resource run_states=run_states/>
                                    <Chart run_ids=run_ids run_states=run_states/>
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
