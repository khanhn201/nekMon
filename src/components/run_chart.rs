use leptos::prelude::*;

use crate::model::{Run, Project, Server};
use crate::components::run_list::get_run_records;
use crate::log_parser::Record;
use crate::vega_lite::vega_embed;

#[component]
pub fn RunChart(run_id: i64) -> impl IntoView {
    let records_resource = LocalResource::new(move || async move {
        get_run_records(run_id).await
    });

    view! {
        <div class="flex flex-col gap-2 p-4 grow">
            <Transition fallback=move || view! { <span class="text-slate-400 text-sm">"Loading records..."</span> }>
                {move || match records_resource.get() {
                    Some(Ok(records)) if !records.is_empty() => {
                        // Serialize to JSON for passing to JS
                        // let json = serde_json::to_string(&records).unwrap_or_default();
                        view! { <VegaChart records=records/> }.into_any()
                    }
                    Some(Ok(_)) => view! { <span class="text-slate-400 text-sm">"No records found"</span> }.into_any(),
                    Some(Err(e)) => view! { <span class="text-red-400 text-sm">"Error: "{e.to_string()}</span> }.into_any(),
                    None => view! { <span></span> }.into_any(),
                }}
            </Transition>
        </div>
    }
}

#[component]
fn VegaChart(records: Vec<Record>) -> impl IntoView {
    let container: NodeRef<leptos::html::Div> = NodeRef::new();
    let refresh = RwSignal::new(0u32);

    
    Effect::new(move |_| {
        // refresh.get();
        let Some(el) = container.get() else { return };
        let spec = serde_json::json!({
            "$schema": "https://vega.github.io/schema/vega-lite/v6.json",
            "data": { "values": records },
            "transform": [
                { "filter": "datum.step > 0" }
            ],
            "concat": [
                // {
                //     "mark": "line",
                //     "encoding": {
                //         "x": { "field": "step", "type": "quantitative", "title": "Step" },
                //         "y": { "field": "dt", "type": "quantitative", "title": "dt" }
                //     }
                // },
                // {
                //     "mark": "line",
                //     "encoding": {
                //         "x": { "field": "step", "type": "quantitative", "title": "Step" },
                //         "y": { "field": "cfl", "type": "quantitative", "title": "CFL" }
                //     }
                // },
                // {
                //     "mark": "line",
                //     "encoding": {
                //         "x": { "field": "step", "type": "quantitative", "title": "Step" },
                //         "y": { "field": "solve_time", "type": "quantitative", "title": "Solve Time (s)" }
                //     }
                // },
                // {
                //     "mark": "line",
                //     "encoding": {
                //         "x": { "field": "step", "type": "quantitative", "title": "Step" },
                //         "y": { "field": "gmres_iter", "type": "quantitative", "title": "GMRES Iterations" }
                //     }
                // },
                // {
                //     "mark": "line",
                //     "encoding": {
                //         "x": { "field": "step", "type": "quantitative", "title": "Step" },
                //         "y": { "field": "gmres_residual", "type": "quantitative", "title": "GMRES Residual" }
                //     }
                // },
                {
                    "mark": "line",
                    "encoding": {
                        "x": { "field": "step", "type": "quantitative", "title": "Step" },
                        "y": { 
                            "field": "cum_error",
                            "type": "quantitative",
                            "title": "Cumulative Error",
                            "scale": {
                                "type": "log"
                            }
                        }
                    }
                },
                // {
                //     "mark": "line",
                //     "encoding": {
                //         "x": { "field": "step", "type": "quantitative", "title": "Step" },
                //         "y": { "field": "cum_gr", "type": "quantitative", "title": "Cumulative Growth" }
                //     }
                // }
            ],
            "columns": 2,
            "resolve": { "scale": { "x": "shared" } }
        });
 
        vega_embed(el.into(), &spec.to_string());
        // set_timeout(
        //     move || refresh.update(|n| *n += 1),
        //     std::time::Duration::from_millis(50000),
        // );

    });
        
    view! {
        <div class="flex" node_ref=container />
        // <div id="vis" />
    }
}
