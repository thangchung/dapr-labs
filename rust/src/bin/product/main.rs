use std::{time::Duration, env};

use axum::{
    error_handling::HandleErrorLayer,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use clap::Parser;
use serde::Serialize;
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Parser)]
struct Config {
    #[clap(default_value = "localhost", env)]
    host: String,
    #[clap(default_value = "5001", env)]
    app_port: u16,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ItemType {
    name: String,
    item_type: i8,
    price: f32,
    image: String,
}

#[derive(Clone)]
struct AppState {
    item_types: Vec<ItemType>,
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "debug");
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "product_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = AppState {
        item_types: vec![
            ItemType {
                name: "CAPPUCCINO".to_string(),
                item_type: 0,
                price: 4.5,
                image: "img/CAPPUCCINO.png".to_string(),
            },
            ItemType {
                name: "COFFEE_BLACK".to_string(),
                item_type: 1,
                price: 3.0,
                image: "img/COFFEE_BLACK.png".to_string(),
            },
            ItemType {
                name: "COFFEE_WITH_ROOM".to_string(),
                item_type: 2,
                price: 3.0,
                image: "img/COFFEE_WITH_ROOM.png".to_string(),
            },
            ItemType {
                name: "ESPRESSO".to_string(),
                item_type: 3,
                price: 3.5,
                image: "img/ESPRESSO.png".to_string(),
            },
            ItemType {
                name: "ESPRESSO_DOUBLE".to_string(),
                item_type: 4,
                price: 4.5,
                image: "img/ESPRESSO_DOUBLE.png".to_string(),
            },
            ItemType {
                name: "LATTE".to_string(),
                item_type: 5,
                price: 4.5,
                image: "img/LATTE.png".to_string(),
            },
            ItemType {
                name: "CAKEPOP".to_string(),
                item_type: 6,
                price: 2.5,
                image: "img/CAKEPOP.png".to_string(),
            },
            ItemType {
                name: "CROISSANT".to_string(),
                item_type: 7,
                price: 3.25,
                image: "img/CROISSANT.png".to_string(),
            },
            ItemType {
                name: "MUFFIN".to_string(),
                item_type: 8,
                price: 3.0,
                image: "img/MUFFIN.png".to_string(),
            },
            ItemType {
                name: "CROISSANT_CHOCOLATE".to_string(),
                item_type: 9,
                price: 3.5,
                image: "img/CROISSANT_CHOCOLATE.png".to_string(),
            },
        ],
    };

    let app = Router::new()
        .route("/", get(home_handler))
        .route("/v1/api/item-types", get(item_types_handler))
        .route("/v1/api/items-by-types/:types", get(item_by_types_handler))
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

    let config = Config::parse();
    let addr: String = format!("{}:{}", config.host.as_str(), config.app_port);

    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn item_by_types_handler(
    Path(types): Path<String>,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let mut temp: Vec<ItemType> = Vec::new();

    for i in app.item_types {
        let parts = types.split(',');

        let ii = i.clone();
        for j in parts {
            if ii.item_type.to_string().as_str() == j {
                temp.push(ii.clone())
            }
        }
    }

    Json(temp)
}

async fn item_types_handler(State(app): State<AppState>) -> impl IntoResponse {
    Json(app.item_types)
}

async fn home_handler() -> impl IntoResponse {
    StatusCode::OK
}
