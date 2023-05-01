use std::sync::Arc;

use axum::{routing::get, Json, Router};
use slog::{info, o, Fuse, Logger};

pub mod logs;

struct AppState {
    logger: Logger,
}

#[tokio::main]
async fn main() {
    let log: Logger = Logger::root(Fuse(logs::PrintlnDrain), o!("slog" => true));
    dotenv::dotenv().ok();

    let app = Router::new()
        .route("/", get(handler))
        .with_state(Arc::new(AppState {
            logger: log.clone(),
        }));
    let addr: &str = "0.0.0.0:5000";

    info!(log, "listening on {addr}", addr = addr);

    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> Json<&'static str> {
    Json("Product API")
}
