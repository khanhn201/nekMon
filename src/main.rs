use axum::{
    response::{self, IntoResponse},
    routing::get,
    Router,
};

use leptos::logging::log;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};

use nekMon::app::*;
use nekMon::app_state::AppState;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;

    let routes = generate_route_list(App);
    let app_state = AppState::new().await.unwrap();

    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            move || provide_context(app_state.clone()),
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
