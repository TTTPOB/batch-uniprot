use axum::{
    extract::Json,
    routing::{get, post},
    Router,
};
use backend_rs::uniprot_proxy::{self, make_query};
use env_logger;

#[tokio::main]
async fn main() {
    env_logger::init();
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route(
            "/idmapping/v1",
            post(
                |Json(q): Json<uniprot_proxy::Query>| async move { make_query(&q).await.unwrap() },
            ),
        );
    eprintln!("Starting server at http://127.0.0.1:3000");
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
    eprintln!("Server started on http://127.0.0.1:3000")
}
