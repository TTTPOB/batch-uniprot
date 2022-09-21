use axum::{
    extract::Json,
    routing::{get, post},
    Router,
};
use backend_rs::uniprot_proxy::{self, make_query};
use env_logger;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;

#[tokio::main]
async fn main() {
    env_logger::init();

    // add a compression layer using gzip encoding
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route(
            "/idmapping/v1",
            post(
                |Json(q): Json<uniprot_proxy::Query>| async move { make_query(&q).await.unwrap() },
            ),
        )
        .layer(ServiceBuilder::new().layer(CompressionLayer::new()));
    eprintln!("Starting server at http://127.0.0.1:3000");

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
    eprintln!("Server started on http://127.0.0.1:3000")
}
