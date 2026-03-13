use axum::{
    response::{self, IntoResponse},
    routing::get,
    Router,
};

use leptos::logging::log;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};

use async_graphql::{http::GraphiQLSource, EmptySubscription, Schema};
use async_graphql_axum::GraphQL;

use nekMon::app::*;
use nekMon::app_state::AppState;
use nekMon::schema::*;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;

    let routes = generate_route_list(App);
    for route in &routes {
        leptos::logging::log!("{}", route.path());
    }
    let app_state = AppState::new().await.unwrap();

    async fn graphiql() -> impl IntoResponse {
        response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
    }
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(app_state.clone())
        .finish();
    let graphql_handler = GraphQL::new(schema);

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
        .with_state(leptos_options)
        .route("/graphql", get(graphiql).post_service(graphql_handler));

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
