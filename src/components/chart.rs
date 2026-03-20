use leptos::prelude::*;

use crate::models::run::*;
use crate::vega_lite::*;

use std::collections::HashMap;
use std::collections::HashSet;

#[component]
pub fn Chart(
    run_ids: Vec<i64>,
    colors: HashMap<i64, &'static str>,
    hovered_id: RwSignal<Option<i64>>,
    hidden_ids: RwSignal<HashSet<i64>>,
) -> impl IntoView {
    let vega_view: StoredValue<Option<VegaView>> = StoredValue::new(None);
    let all_records: RwSignal<HashMap<i64, Vec<serde_json::Value>>> = RwSignal::new(HashMap::new());
    let to_be_updated: RwSignal<HashMap<i64, bool>> = RwSignal::new(HashMap::new());
    let color_domain: Vec<String> = run_ids.iter()
        .map(|id| id.to_string())
        .collect();
    let color_range: Vec<&str> = run_ids.iter()
        .map(|id| colors.get(id).copied().unwrap_or("#94a3b8"))
        .collect();


    let available_fields: RwSignal<Vec<String>> = RwSignal::new(vec![]);

    let x_field: RwSignal<Option<String>> = RwSignal::new(None);
    let y_field: RwSignal<Option<String>> = RwSignal::new(None);
    let y_scale = RwSignal::new("log");

    Effect::new(move |_| {
        let fields = available_fields.get();
        if fields.is_empty() { return; }
        if x_field.get_untracked().is_none() {
            x_field.set(Some("time".to_string()));
        }
        if y_field.get_untracked().is_none() {
            y_field.set(fields.get(1).or(fields.first()).cloned());
        }
    });


    let refresh = RwSignal::new(0u32);
    Effect::new(move |_| {
        let _ = refresh.get();
        set_timeout(
            move || refresh.update(|n| *n += 1),
            std::time::Duration::from_millis(10_000),
        );
    });

    let _resources: Vec<_> = run_ids.iter().map(|&id| {
        let prev_records: StoredValue<Option<Vec<serde_json::Value>>> = StoredValue::new(None);

        let resource = LocalResource::new(move || {
            let _ = refresh.get();
            async move { get_run_records(id).await }
        });
        Effect::new(move |_| {
            if let Some(Ok(records)) = resource.get() {
                let tagged: Vec<serde_json::Value> = records.iter().filter_map(|r| {
                    let mut v = serde_json::to_value(r).ok()?;
                    v.as_object_mut()?.insert("run_id".into(), serde_json::json!(id.to_string()));
                    Some(v)
                }).collect();
                let prev = prev_records.get_value();
                if prev.as_deref() == Some(tagged.as_slice()) {
                    to_be_updated.update(|map| { map.insert(id, false); });
                } else {
                    to_be_updated.update(|map| { map.insert(id, true); });
                }
                all_records.update(|map| { map.insert(id, tagged.clone()); });

                if available_fields.get_untracked().is_empty() {
                    if let Some(first) = tagged.first() {
                        if let Some(obj) = first.as_object() {
                            let fields: Vec<String> = obj.keys()
                                .filter(|k| *k != "run_id") // exclude non-plottable
                                .cloned()
                                .collect();
                            available_fields.set(fields);
                        }
                    }
                }
                prev_records.set_value(Some(tagged));
            }
        });
        resource
    }).collect();
    

    let run_ids_clone = run_ids.clone();
    Effect::new(move |_| {
        let updates = to_be_updated.get();
        if updates.len() < run_ids_clone.clone().len() { return; }
        
        let records_map = all_records.get_untracked();
        vega_view.with_value(|v| {
            if let Some(view) = v {
                let mut any_updated = false;
                for (id, needs_update) in &updates {
                    if *needs_update {
                        if let Some(records) = records_map.get(id) {
                            vega_set_data(view, &format!("run_{id}"), records);
                            any_updated = true;
                        }
                    }
                }
                if any_updated {
                    vega_run(view);
                }
            }
        });

        to_be_updated.update(|map| map.clear());
    });

    Effect::new(move |_| {
        let hovered = hovered_id.get().map(|id| id.to_string());
        vega_view.with_value(|v| {
            if let Some(view) = v {
                vega_set_signal(view, "hoveredId", hovered.as_deref());
            }
        });
    });

    Effect::new(move |_| {
        let hidden: Vec<i64> = hidden_ids.get().iter().copied().collect();
        vega_view.with_value(|v| {
            if let Some(view) = v {
                vega_set_signal_array(view, "hiddenIds", &hidden);
            }
        });
    });


    let container: NodeRef<leptos::html::Div> = NodeRef::new();
    let run_ids_clone = run_ids.clone();
    Effect::new(move |_| {
        let Some(el) = container.get() else { return };
        
        let layers: Vec<serde_json::Value> = run_ids_clone.iter().map(|id| {
            serde_json::json!({
                "data": { "name": format!("run_{id}") },
                "transform": [
                    { "filter": "datum.step > 20" },
                    { "filter": format!("indexof(hiddenIds, datum.run_id) < 0") }
                ],
                "mark": { "type": "line", "tooltip": true },
                "encoding": {
                    "x": { "field": x_field, "type": "quantitative" },
                    "y": {
                        "field": y_field.get(),
                        "type": "quantitative",
                        "scale": { "type": y_scale.get() }
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
                        "condition": {
                            "test": "hoveredId === null || datum.run_id === hoveredId",
                            "value": 1.0
                        },
                        "value": 0.15
                    }
                }
            })
        }).collect();
        let spec = serde_json::json!({
            "$schema": "https://vega.github.io/schema/vega-lite/v6.json",
            "width": "container",
            "height": "container",
            "autosize": { "type": "fit", "contains": "padding" },
            "params": [
                { "name": "hoveredId", "value": null },
                { "name": "hiddenIds", "value": [] }
            ],
            "layer": layers
        }).to_string();
        let current_records = all_records.get_untracked();
        let current_hidden: Vec<i64> = hidden_ids.get_untracked().iter().copied().collect();
        let current_hovered = hovered_id.get_untracked().map(|id| id.to_string());

        vega_embed(el.into(), &spec, move |view| {
            // Push all current records into the new view
            for (id, records) in &current_records {
                vega_set_data(&view, &format!("run_{id}"), records);
            }
            // Restore signal state
            vega_set_signal(&view, "hoveredId", current_hovered.as_deref());
            vega_set_signal_array(&view, "hiddenIds", &current_hidden);
            vega_run(&view);
            vega_view.set_value(Some(view));
        });
    });

    view! {
        <div class="flex flex-col size-full p-4 min-w-0">
            <div class="flex gap-4 px-4 pt-3 pb-1 text-sm text-slate-600 items-center">
                <label class="flex items-center gap-1">
                    "X"
                    <select
                        class="border rounded px-1 py-0.5 bg-white"
                        on:change=move |e| x_field.set(Some(event_target_value(&e)))
                    >
                        {move || ["time", "step"].into_iter().map(|f| {
                            let selected = x_field.get().as_deref() == Some(&f);
                            view! { <option value=f selected=selected>{f}</option> }
                        }).collect_view()}
                    </select>
                </label>
                <label class="flex items-center gap-1">
                    "Y"
                    <select
                        class="border rounded px-1 py-0.5 bg-white"
                        on:change=move |e| y_field.set(Some(event_target_value(&e)))
                    >
                        {move || available_fields.get().into_iter().map(|f| {
                            let selected = y_field.get().as_deref() == Some(&f);
                            view! { <option value=f.clone() selected=selected>{f.clone()}</option> }
                        }).collect_view()}
                    </select>
                </label>
                <label class="flex items-center gap-1">
                    "Scale"
                    <select
                        class="border rounded px-1 py-0.5 bg-white"
                        on:change=move |e| y_scale.set(
                            if event_target_value(&e) == "log" { "log" } else { "linear" }
                        )
                    >
                        <option value="log">"Log"</option>
                        <option value="linear">"Linear"</option>
                    </select>
                </label>
            </div>
            <div class="size-full min-w-0" node_ref=container />
        </div>
    }
}
