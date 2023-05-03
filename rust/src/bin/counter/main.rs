use std::{env, time::Duration};

use chrono::prelude::*;

use axum::{
    error_handling::HandleErrorLayer,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use counter_entity::{line_items, orders, orders::Entity as Order};
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set};
use serde::Deserialize;
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[derive(Debug, Parser)]
struct Config {
    #[clap(default_value = "0.0.0.0", env)]
    host: String,
    #[clap(default_value = "5002", env)]
    app_port: u16,
    #[clap(default_value = "postgres://postgres:P@ssw0rd@127.0.0.1/postgres", env)]
    database_url: String,
}

#[derive(Clone)]
struct AppState {
    db_conn: DatabaseConnection,
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "debug");
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "counter_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::parse();

    let db_conn: DatabaseConnection = Database::connect(config.database_url)
        .await
        .expect("Database connection failed");

    let state = AppState { db_conn };

    let app = Router::new()
        .route("/", get(home_handler))
        .route("/v1/api/orders", post(place_order_handler))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {}", error),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(10))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                        .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
                )
                .into_inner(),
        )
        .with_state(state);

    let addr: String = format!("{}:{}", config.host.as_str(), config.app_port);

    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Debug, Deserialize)]
struct PlaceOrderItem {
    item_type: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlaceOrder {
    command_type: Option<i32>,
    order_source: Option<i32>,
    location: Option<i32>,
    loyalty_member_id: Option<Uuid>,
    barista_items: Option<Vec<PlaceOrderItem>>,
    kitchen_items: Option<Vec<PlaceOrderItem>>,
    timestamp: Option<DateTime<Utc>>,
}

async fn place_order_handler(
    State(app): State<AppState>,
    Json(input): Json<PlaceOrder>,
) -> impl IntoResponse {
    let result = orders::ActiveModel {
        order_source: Set(input.order_source.unwrap_or(0)),
        loyalty_member_id: Set(input.loyalty_member_id.unwrap_or_default()),
        order_status: Set(1),
        ..Default::default()
    }
    .save(&app.db_conn)
    .await;

    //todo
    let r = match result {
        Ok(_) => "",
        Err(_) => ""
    };

    // for barista_item in input.barista_items {
    //     line_items::ActiveModel {
    //         id: Set(Uuid::new_v4()),
    //         name: Set(barista_item.item_type),
    //         ..Default::default()
    //     }
    // }

    match Order::find().all(&app.db_conn).await {
        Ok(_) => (StatusCode::CREATED, Json("created".to_string())),
        Err(_) => (StatusCode::BAD_REQUEST, Json("err".to_string())),
    }
}

async fn home_handler() -> impl IntoResponse {
    StatusCode::OK
}
