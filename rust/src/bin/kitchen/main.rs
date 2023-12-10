use std::time;
use std::{env, time::Duration};

use chrono::serde::ts_seconds::deserialize as from_ts;
use chrono::{prelude::*, serde::ts_seconds};

use axum::{
    error_handling::HandleErrorLayer,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use cloudevents::Event;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, Set};
use serde::{Deserialize, Serialize};

use kitchen_entity::kitchen_orders;
use serde_json::json;
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

// App config
#[derive(Debug, Parser, Clone)]
struct Config {
    #[clap(default_value = "localhost", env)]
    host: String,
    #[clap(default_value = "5004", env)]
    app_port: u16,
    #[clap(default_value = "postgres://localhost/db", env)]
    database_url: String,
    #[clap(default_value = "http://localhost:3500", env)]
    dapr_url: String,
}

#[derive(Clone)]
struct AppState {
    config: Config,
    db_conn: DatabaseConnection,
}

// Command, Query and Models
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SubscribeModel {
    pubsubname: String,
    topic: String,
    route: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KitchenOrderIn {
    pub order_id: Uuid,
    pub item_line_id: Uuid,
    pub item_type: i32,
    #[serde(deserialize_with = "from_ts")]
    pub time_in: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct KitchenOrderUp {
    pub order_id: Uuid,
    pub item_line_id: Uuid,
    pub name: String,
    pub item_type: i32,
    #[serde(with = "ts_seconds")]
    pub time_in: DateTime<Utc>,
    pub made_by: String,
    #[serde(with = "ts_seconds")]
    pub time_up: DateTime<Utc>,
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "debug");
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kitchen_api=debug,tower_http=debug".into()),
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

async fn home_handler() -> impl IntoResponse {
    StatusCode::OK
}

async fn get_subscribe_handler() -> impl IntoResponse {
    let subscribe_model = vec![SubscribeModel {
        pubsubname: "kitchenpubsub".to_string(),
        topic: "kitchenordered".to_string(),
        route: "place-order".to_string(),
    }];

    (StatusCode::OK, Json(subscribe_model))
}

async fn place_order_handler(
    State(app): State<AppState>,
    Json(event): Json<Event>,
) -> impl IntoResponse {
    tracing::debug!("kitchen_order_in_event: {:?}", event.data());

    let event = match event.data() {
        Some(cloudevents::Data::Json(value)) => {
            let temp = <KitchenOrderIn as Deserialize>::deserialize(value);
            tracing::debug!("KitchenOrderIn: {:?}", temp);
            temp.unwrap()
        }
        _ => unreachable!(),
    };

    let tz = calculate_delay(event.item_type).await;

    let result = kitchen_orders::ActiveModel {
        order_id: Set(event.item_line_id),
        item_name: Set("name".to_string()), //todo
        item_type: Set(event.item_type),
        created: Set(event.time_in.with_timezone(&tz)),
        time_up: Set(Utc::now().with_timezone(&tz)),
        ..Default::default()
    }
    .save(&app.db_conn)
    .await
    .unwrap();

    // publish domain event
    publish_kitchen_order_up_event(
        &app.config.dapr_url,
        "kitchenorderuppubsub",
        "kitchenorderup",
        KitchenOrderUp {
            order_id: event.order_id,
            item_line_id: event.item_line_id,
            name: result.item_name.clone().take().unwrap(),
            item_type: result.item_type.clone().take().unwrap(),
            time_in: result.created.clone().take().unwrap().into(),
            made_by: "tc".to_string(),
            time_up: result.time_up.clone().take().unwrap().into(),
        },
    )
    .await;

    (StatusCode::CREATED, Json(()))
}

async fn publish_kitchen_order_up_event(
    dapr_url: &str,
    pubsub_name: &str,
    topic: &str,
    event: KitchenOrderUp,
) {
    let url = format!("{}/v1.0/publish/{}/{}", dapr_url, pubsub_name, topic);
    tracing::debug!("url: {}", url);

    surf::post(url).body(json!(event)).await.unwrap();
}

async fn calculate_delay(item_type: i32) -> FixedOffset {
    let max_seconds = match item_type {
        6 => 7,   // CROISSANT
        7 => 7,   // CROISSANT_CHOCOLATE
        8 => 5,   // CAKEPOP
        9.. => 7, // MUFFIN
        _ => 5,   // others
    };

    // emulate the delay
    // let random_seconds = thread_rng().gen_range(1..max_seconds);
    let random_duration = time::Duration::from_secs(max_seconds);
    tracing::debug!("sleeping in {} seconds. zzzzzz", max_seconds);
    tokio::time::sleep(random_duration).await;
    FixedOffset::east_opt(max_seconds as i32).unwrap()
}
