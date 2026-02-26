use axum::{
    Router,
    response::{self, IntoResponse},
    routing::get,
};

use leptos::logging::log;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};

use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use async_graphql::{EmptySubscription, Schema, http::GraphiQLSource};
use async_graphql_axum::GraphQL;

use nekMon::schema::*;
use nekMon::app::*;
use nekMon::ssh::*;


#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    let _ssh_client = SSHClient::new().await.unwrap();
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:nekMon.db?mode=rwc") // TODO: configurable
        .await.unwrap();
    
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;

    let routes = generate_route_list(App);

    async fn graphiql() -> impl IntoResponse {
        response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
    }
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(pool)
        .finish();
    let graphql_handler = GraphQL::new(schema);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
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
