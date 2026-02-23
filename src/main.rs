
use async_graphql::{EmptyMutation, EmptySubscription, Schema, http::GraphiQLSource};
use async_graphql_axum::GraphQL;
use axum::{
    Router,
    response::{self, IntoResponse},
    routing::get,
};
use tokio::net::TcpListener;

mod model;
use model::*;

async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/").finish())
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use nekMon::app::*;

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    // let app = Router::new()
    //     .leptos_routes(&leptos_options, routes, {
    //         let leptos_options = leptos_options.clone();
    //         move || shell(leptos_options.clone())
    //     })
    //     .fallback(leptos_axum::file_and_error_handler(shell))
    //     .with_state(leptos_options);
    //
    // // run our app with hyper
    // // `axum::Server` is a re-export of `hyper::Server`
    // log!("listening on http://{}", &addr);
    // let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    // axum::serve(listener, app.into_make_service())
    //     .await
    //     .unwrap();
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(Project::new())
        .finish();

    let app = Router::new().route("/", get(graphiql).post_service(GraphQL::new(schema)));

    println!("GraphiQL IDE: http://localhost:8000");

    axum::serve(TcpListener::bind("127.0.0.1:8000").await.unwrap(), app)
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
