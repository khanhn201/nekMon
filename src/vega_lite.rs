#[cfg(not(feature = "ssr"))]
mod csr {
    use wasm_bindgen::{prelude::wasm_bindgen, JsValue};


    // #[wasm_bindgen(module = "https://cdn.jsdelivr.net/npm/vega@6.2.0/+esm")]
    // extern "C" {}
    //
    // #[wasm_bindgen(module = "https://cdn.jsdelivr.net/npm/vega-lite@6.4.2/+esm")]
    // extern "C" {}

    // Comes with vega and vega-lite
    #[wasm_bindgen(module = "https://cdn.jsdelivr.net/npm/vega-embed@7.1.0/+esm")] 
    extern "C" {
        #[wasm_bindgen(js_name = default)]
        fn embed(el: &web_sys::Element, spec: &JsValue);
    }
    
    pub fn vega_embed(el: web_sys::Element, spec: &str) {
        let js_spec = js_sys::JSON::parse(spec).unwrap();

        embed(&el, &js_spec);

        
    }
}

#[cfg(feature = "ssr")]
mod ssr {
    // noop under ssr
    pub fn vega_embed(_el: web_sys::Element, _spec: &str) {}
}

#[cfg(not(feature = "ssr"))]
pub use csr::*;
#[cfg(feature = "ssr")]
pub use ssr::*;

