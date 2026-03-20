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
    let run_ids_clone = run_ids.clone();
    let all_records: RwSignal<HashMap<i64, Vec<serde_json::Value>>> = RwSignal::new(HashMap::new());
    let _resources: Vec<_> = run_ids.iter().map(|&id| {
        let refresh = RwSignal::new(0u32);
        let resource = LocalResource::new(move || {
            let _ = refresh.get();
            async move {
                let result = get_run_records(id).await;
                // set_timeout(
                //     move || refresh.update(|n| *n += 1),
                //     std::time::Duration::from_millis(10_000),
                // );
                result
            }
        });
        Effect::new(move |_| {
            if let Some(Ok(records)) = resource.get() {
                let tagged: Vec<serde_json::Value> = records.iter().filter_map(|r| {
                    let mut v = serde_json::to_value(r).ok()?;
                    v.as_object_mut()?.insert("run_id".into(), serde_json::json!(id.to_string()));
                    Some(v)
                }).collect();
                all_records.update(|map| { map.insert(id, tagged); });
            }
        });
        resource
    }).collect();


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
        let records_map = all_records.get();
        if records_map.is_empty() { return; }
        
        let color_domain: Vec<String> = run_ids.iter()
            .map(|id| id.to_string())
            .collect();
        let color_range: Vec<&str> = run_ids.iter()
            .map(|id| colors.get(id).copied().unwrap_or("#94a3b8"))
            .collect();
        let records: Vec<serde_json::Value> = run_ids.iter()
            .filter_map(|id| records_map.get(id))
            .flatten()
            .cloned()
            .collect();
        let spec = serde_json::json!({
            "$schema": "https://vega.github.io/schema/vega-lite/v6.json",
            "data": { "values": records },
            "width": "container",
            "height": "container",
            "autosize": { "type": "fit", "contains": "padding" },
            "params": [
                { "name": "hoveredId", "value": null },
                { "name": "hiddenIds", "value": [] }
            ],
            "transform": [
                { "filter": "datum.step > 20" },
                { "filter": "indexof(hiddenIds, datum.run_id) < 0" }
            ],
            "mark": {
                "type": "line",
                "tooltip": true
            },
            "encoding": {
                "x": { "field": "time", "type": "quantitative" },
                "y": {
                    "field": "cum_error",
                    "type": "quantitative",
                    "scale": {
                        "type": "log"
                    }
                },
                "color": {
                    "field": "run_id",
                    "type": "nominal",
                    "scale": { "domain": color_domain, "range": color_range },
                    "legend": null
                },
                "opacity": {
                    "condition": {
                        "test": "hoveredId === null || datum.run_id === hoveredId",
                        "value": 1.0
                    },
                    "value": 0.15
                }
            },
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
