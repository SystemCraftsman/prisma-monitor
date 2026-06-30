use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use tower_http::services::ServeDir;

use prisma_monitor::web;

#[tokio::main]
async fn main() {
    let state = web::new_shared_state();

    let app = Router::new()
        .route("/", get(web::routes::dashboard))
        .route("/upload", get(web::routes::upload_page))
        .route("/api/upload", post(web::routes::handle_upload))
        .route("/sessions", get(web::routes::sessions_page))
        .route("/sessions/{key}", get(web::routes::session_detail))
        .route("/config", get(web::routes::config_page))
        .nest_service("/static", ServeDir::new("static"))
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    println!("Prisma Monitor running at http://localhost:3000");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
