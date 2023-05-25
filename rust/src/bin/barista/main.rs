use std::{env, time::Duration};

use chrono::prelude::*;
use chrono::serde::ts_seconds::deserialize as from_ts;

use axum::{
    error_handling::HandleErrorLayer,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use cloudevents::Event;
use clap::Parser;
use sea_orm::{
    ActiveModelTrait, Database, DatabaseConnection, Set,
};
use serde::{Deserialize, Serialize};

use tower::{BoxError, ServiceBuilder};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;
use barista_entity::{barista_orders};

#[derive(Debug, Parser, Clone)]
struct Config {
    #[clap(default_value = "localhost", env)]
    host: String,
    #[clap(default_value = "5003", env)]
    app_port: u16,
    #[clap(default_value = "postgres://postgres:P@ssw0rd@127.0.0.1/postgres", env)]
    database_url: String,
    #[clap(default_value = "http://localhost:3500", env)]
    dapr_url: String,
}

#[derive(Clone)]
struct AppState {
    config: Config,
    db_conn: DatabaseConnection,
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "debug");
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "barista_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::parse();

    let db_conn: DatabaseConnection = Database::connect(&config.database_url)
        .await
        .expect("Database connection failed");

    let state = AppState {
        config: config.clone(),
        db_conn,
    };

    let app = Router::new()
        .route("/", get(home_handler))
        .route("/dapr/subscribe", get(get_subscribe_handler))
        .route("/place-order", post(place_order_handler))
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

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SubscribeModel {
    pubsubname: String,
    topic: String,
    route: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BaristaOrderIn {
    pub order_id: Uuid,
    pub item_line_id: Uuid,
    pub item_type: i32,
    #[serde(deserialize_with = "from_ts")]
    pub time_in: DateTime<Utc>,
}

async fn home_handler() -> impl IntoResponse {
    StatusCode::OK
}

async fn get_subscribe_handler() -> impl IntoResponse {
    let temp = vec![SubscribeModel {
        pubsubname: "baristapubsub".to_string(),
        topic: "baristaordered".to_string(),
        route: "place-order".to_string(),
    }];

    (StatusCode::OK, Json(temp))
}

async fn place_order_handler(
    State(app): State<AppState>,
    Json(event): Json<Event>,
) -> impl IntoResponse {
    tracing::debug!("barista_order_in_event: {:?}", event.data());

    let event = match event.data() {
        Some(cloudevents::Data::Json(value)) => {
            let temp = <BaristaOrderIn as Deserialize>::deserialize(value);
            tracing::debug!("BaristaOrderIn: {:?}", temp);
            temp.unwrap()
        },
        _ => unreachable!(),
    };

    _ = barista_orders::ActiveModel {
        order_id: Set(event.item_line_id),
        item_name: Set("name".to_string()), //todo
        item_type: Set(event.item_type),
        created: Set(event.time_in.with_timezone(&FixedOffset::east_opt(0).unwrap())),
        time_up: Set(Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap())), //todo: set delay
        ..Default::default()
    }
    .save(&app.db_conn)
    .await
    .unwrap();

    (StatusCode::CREATED, Json(()))
}
