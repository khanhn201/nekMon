use leptos::prelude::*;

use crate::models::run::*;
use crate::vega_lite::vega_embed;
use crate::components::project_view::RunState;

use std::collections::HashMap;
#[component]
pub fn Chart(
    run_ids: Vec<i64>,
    run_states: RwSignal<HashMap<i64, RunState>>,
) -> impl IntoView {
    let refresh = RwSignal::new(0u32);
    // let records_resource = LocalResource::new(move || async move { get_run_records(run_ids).await });
    let run_ids_clone = run_ids.clone();
    let records_resource = LocalResource::new(move || {
        let _ = refresh.get();
        let run_ids = run_ids_clone.clone();
        async move {
            let results = futures::future::join_all(
                run_ids.clone().iter().map(|&id| get_run_records(id))
            ).await;
            set_timeout(
                move || refresh.update(|n| *n += 1),
                std::time::Duration::from_millis(10_000),
            );
            results
        }
    });

    view! {
        <div class="flex size-full p-4 min-w-0">
            <Transition fallback=move || view! { <span class="text-slate-400 text-sm">"Loading records..."</span> }>
                {move || {
                    let Some(results) = records_resource.get() else {
                        return view! { <span></span> }.into_any();
                    };
                    let run_states = run_states.get();
                    let any_hovered = run_states.values().any(|s| s.hovered.get());

                    let color_domain: Vec<String> = run_ids.iter()
                        .filter(|id| run_states.get(id).map(|s| s.visible.get()).unwrap_or(true))
                        .map(|id| id.to_string())
                        .collect();
                    let color_range: Vec<&str> = run_ids.iter()
                        .filter(|id| run_states.get(id).map(|s| s.visible.get()).unwrap_or(true))
                        .map(|id| run_states.get(id).map(|s| s.color).unwrap_or("#94a3b8"))
                        .collect();

                    let tagged: Result<Vec<serde_json::Value>, _> = run_ids.iter()
                        .zip(results)
                        .filter(|(id, _)| run_states.get(id).map(|s| s.visible.get()).unwrap_or(true))
                        .flat_map(|(id, result)| match result {
                            Ok(records) => records.into_iter().map(|r| {
                                let is_hovered = run_states.get(id).map(|s| s.hovered.get()).unwrap_or(false);
                                let opacity = if any_hovered {
                                    if is_hovered { 1.0 } else { 0.15 }
                                } else {
                                    1.0
                                };
                                let mut v = serde_json::to_value(&r).unwrap_or_default();
                                if let Some(obj) = v.as_object_mut() {
                                    obj.insert("run_id".into(), serde_json::json!(id.to_string()));
                                    obj.insert("opacity".into(), serde_json::json!(opacity));
                                }
                                Ok(v)
                            }).collect::<Vec<_>>(),
                            Err(e) => vec![Err(e)],
                        })
                        .collect();
                    match tagged {
                        Err(e) => view! {
                            <span class="text-red-400 text-sm">"Error: "{e.to_string()}</span>
                        }.into_any(),
                        Ok(records) => view! {
                            <VegaChart records=records color_domain=color_domain color_range=color_range/>
                        }.into_any(),
                    }
                }}
            </Transition>
        </div>
    }
}

#[component]
fn VegaChart(
    records: Vec<serde_json::Value>,
    color_domain: Vec<String>,
    color_range: Vec<&'static str>,
) -> impl IntoView {
    let container: NodeRef<leptos::html::Div> = NodeRef::new();
    
    Effect::new(move |_| {
        let Some(el) = container.get() else { return };
        let spec = serde_json::json!({
            "$schema": "https://vega.github.io/schema/vega-lite/v6.json",
            "data": { "values": records },
            "width": "container",
            "height": "container",
            "autosize": {
                "type": "fit",
                "contains": "padding"
            },
            "transform": [
                { "filter": "datum.step > 20" }
            ],
            "mark": {
                "type": "line",
                "tooltip": true
            },
            "encoding": {
                "x": { "field": "time", "type": "quantitative" },
                "y": {
                    "field": "cum_error",
                    // "field": "dsum_avg",
                    "type": "quantitative",
                    "scale": {
                        "type": "log"
                    }
                },
                "color": {
                    "field": "run_id",
                    "type": "nominal",
                    "scale": {
                        "domain": color_domain,
                        "range": color_range
                    },
                    "legend": null
                },
                "opacity": {
                    "field": "opacity",
                    "type": "quantitative",
                    "scale": { "domain": [0.0, 1.0], "range": [0.0, 1.0] },
                    "legend": null
                }
            },
        });
        vega_embed(el.into(), &spec.to_string());
    });


    view! {
        <div class="size-full min-w-0" node_ref=container />
    }
}
