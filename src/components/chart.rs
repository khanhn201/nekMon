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
    let container: NodeRef<leptos::html::Div> = NodeRef::new();
    let vega_view: StoredValue<Option<VegaView>> = StoredValue::new(None);
    let all_records: RwSignal<HashMap<i64, Option<Vec<serde_json::Value>>>> = RwSignal::new(HashMap::new());
    let color_domain: Vec<String> = run_ids.iter()
        .map(|id| id.to_string())
        .collect();
    let color_range: Vec<&str> = run_ids.iter()
        .map(|id| colors.get(id).copied().unwrap_or("#94a3b8"))
        .collect();


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
                    leptos::logging::log!("same value detected");
                    all_records.update(|map| { map.insert(id, None); });
                } else {
                    all_records.update(|map| { map.insert(id, Some(tagged.clone())); });
                }
                prev_records.set_value(Some(tagged));
            }
        });
        resource
    }).collect();
    
    let layers: Vec<serde_json::Value> = run_ids.iter().map(|id| {
        serde_json::json!({
            "data": { "name": format!("run_{id}") },
            "transform": [
                { "filter": "datum.step > 20" },
                { "filter": format!("indexof(hiddenIds, datum.run_id) < 0") }
            ],
            "mark": { "type": "line", "tooltip": true },
            "encoding": {
                "x": { "field": "time", "type": "quantitative" },
                "y": {
                    "field": "cum_error",
                    "type": "quantitative",
                    "scale": { "type": "log" }
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

    Effect::new(move |_| {
        let updates = all_records.get();
        let run_ids_clone = run_ids.clone();
        if updates.len() < run_ids_clone.clone().len() { return; }

        vega_view.with_value(|v| {
            if let Some(view) = v {
                for (id, records) in &updates {
                    if let Some(records) = records {
                        vega_set_data(view, &format!("run_{id}"), records);
                    }
                }
                vega_run(view);
            }
        });

        all_records.update(|map| map.clear());
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


    Effect::new(move |_| {
        let Some(el) = container.get() else { return };
        
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
        vega_embed(el.into(), &spec, move |view| {
            vega_view.set_value(Some(view));
        });
    });

    view! {
        <div class="flex size-full p-4 min-w-0">
            <div class="size-full min-w-0" node_ref=container />
        </div>
    }
}
