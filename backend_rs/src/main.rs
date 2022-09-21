use axum::{
    extract::Json,
    routing::{get, post},
    Router,
};
use backend_rs::uniprot_proxy::{make_query, Query};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/api/v1", post(test));
    axum::Server::bind(&"127.0.0.1:3000"
        .parse()
        .unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
    eprintln!("Server started on http://127.0.0.1:3000")
}

async fn test(Json(q): Json<Query>) -> String {
    make_query(&q).await.unwrap()
}
