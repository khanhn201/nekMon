#[cfg(not(feature = "ssr"))]
mod csr {
    use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    #[wasm_bindgen(module = "https://cdn.jsdelivr.net/npm/vega-embed@7.1.0/+esm")]
    extern "C" {
        #[wasm_bindgen(js_name = default)]
        fn embed(el: &web_sys::Element, spec: &JsValue) -> js_sys::Promise;
    }

    // Bind the vega View object's methods directly
    #[wasm_bindgen]
    extern "C" {
        pub type VegaView;

        #[wasm_bindgen(method)]
        pub fn signal(this: &VegaView, name: &str, value: &JsValue) -> VegaView;

        #[wasm_bindgen(method)]
        pub fn run(this: &VegaView);
    }

    // Bind the vegaEmbed result object { view, spec, vgSpec }
    #[wasm_bindgen]
    extern "C" {
        pub type EmbedResult;

        #[wasm_bindgen(method, getter)]
        pub fn view(this: &EmbedResult) -> VegaView;
    }

    pub fn vega_embed(
        el: web_sys::Element,
        spec: &str,
        on_view: impl Fn(VegaView) + 'static,
    ) {
        let js_spec = js_sys::JSON::parse(spec).unwrap();
        let promise = embed(&el, &js_spec);
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(result) = JsFuture::from(promise).await {
                let embed_result: EmbedResult = result.unchecked_into();
                on_view(embed_result.view());
            }
        });
    }

    pub fn vega_set_signal(view: &VegaView, name: &str, value: Option<&str>) {
        let js_value = match value {
            Some(v) => JsValue::from_str(v),
            None => JsValue::NULL,
        };
        view.signal(name, &js_value).run();
    }
    
    pub fn vega_set_signal_array(view: &VegaView, name: &str, value: &[i64]) {
        let array: js_sys::Array = value.iter()
            .map(|id| JsValue::from_str(&id.to_string()))
            .collect();
        view.signal(name, array.as_ref()).run();
    }
}

#[cfg(feature = "ssr")]
mod ssr {
    // Noop for ssr
    pub struct VegaView;

    pub fn vega_embed(_el: web_sys::Element, _spec: &str, _on_view: impl Fn(VegaView) + 'static) {}
    pub fn vega_set_signal(_view: &VegaView, _name: &str, _value: Option<&str>) {}
    pub fn vega_set_signal_array(_view: &VegaView, _name: &str, _value: &[i64]) {}
}

#[cfg(not(feature = "ssr"))]
pub use csr::*;
#[cfg(feature = "ssr")]
pub use ssr::*;
