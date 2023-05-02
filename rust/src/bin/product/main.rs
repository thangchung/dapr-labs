use std::time::Duration;

use axum::{
    error_handling::HandleErrorLayer,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Serialize;
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Serialize, Clone, Copy)]
struct ItemType<'a> {
    name: &'a str,
    item_type: i8,
    price: f32,
    image: &'a str,
}

#[derive(Clone)]
struct AppState<'a> {
    item_types: Vec<ItemType<'a>>,
}

#[tokio::main]
async fn main() {
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
                name: "CAPPUCCINO",
                item_type: 0,
                price: 4.5,
                image: "img/CAPPUCCINO.png",
            },
            ItemType {
                name: "COFFEE_BLACK",
                item_type: 1,
                price: 3.0,
                image: "img/COFFEE_BLACK.png",
            },
            ItemType {
                name: "COFFEE_WITH_ROOM",
                item_type: 2,
                price: 3.0,
                image: "img/COFFEE_WITH_ROOM.png",
            },
            ItemType {
                name: "ESPRESSO",
                item_type: 3,
                price: 3.5,
                image: "img/ESPRESSO.png",
            },
            ItemType {
                name: "ESPRESSO_DOUBLE",
                item_type: 4,
                price: 4.5,
                image: "img/ESPRESSO_DOUBLE.png",
            },
            ItemType {
                name: "LATTE",
                item_type: 5,
                price: 4.5,
                image: "img/LATTE.png",
            },
            ItemType {
                name: "CAKEPOP",
                item_type: 6,
                price: 2.5,
                image: "img/CAKEPOP.png",
            },
            ItemType {
                name: "CROISSANT",
                item_type: 7,
                price: 3.25,
                image: "img/CROISSANT.png",
            },
            ItemType {
                name: "MUFFIN",
                item_type: 8,
                price: 3.0,
                image: "img/MUFFIN.png",
            },
            ItemType {
                name: "CROISSANT_CHOCOLATE",
                item_type: 9,
                price: 3.5,
                image: "img/CROISSANT_CHOCOLATE.png",
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

    let addr: &str = "0.0.0.0:5001";

    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn item_by_types_handler(
    Path(types): Path<String>,
    State(app): State<AppState<'static>>,
) -> impl IntoResponse {
    // tracing::info!("GET: item_by_types_handler");

    let mut temp: Vec<ItemType> = Vec::new();

    for i in app.item_types {
        let parts = types.split(',');

        for j in parts {
            if i.item_type.to_string().as_str() == j {
                temp.push(i)
            }
        }
    }

    Json(temp)
}

async fn item_types_handler(State(app): State<AppState<'static>>) -> impl IntoResponse {
    // tracing::info!("GET: item_types_handler");

    Json(app.item_types.clone())
}

async fn home_handler() -> impl IntoResponse {
    StatusCode::OK
}
