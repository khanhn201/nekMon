#![allow(non_snake_case)]



pub mod app;
pub mod model;
pub mod components;

#[cfg(feature = "ssr")]
pub mod configuration;
#[cfg(feature = "ssr")]
pub mod app_state;
#[cfg(feature = "ssr")]
pub mod schema;
#[cfg(feature = "ssr")]
pub mod ssh;
#[cfg(feature = "ssr")]
pub mod log_parser;



#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
