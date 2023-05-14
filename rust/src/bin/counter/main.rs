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
use counter_entity::{
    line_items,
    line_items::Entity as LineItemEntity,
    orders::Entity as Order,
    orders::{self},
};
use sea_orm::{
    prelude::Decimal,
    sea_query::{Alias, Expr},
    ActiveModelTrait, Database, DatabaseConnection, EntityTrait, JoinType, QuerySelect, Set,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
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
        .route("/v1/api/fulfillment-orders", get(get_order_handler))
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OrderModel {
    pub id: Uuid,
    pub order_source: i32,
    pub loyalty_member_id: Uuid,
    pub order_status: i32,
}

async fn get_order_handler(State(app): State<AppState>) -> impl IntoResponse {
    match Order::find()
        .column_as(
            Expr::tbl(Alias::new("line_items"), line_items::Column::Name).into_simple_expr(),
            "ItemName",
        )
        .join_rev(
            JoinType::InnerJoin,
            LineItemEntity::belongs_to(Order)
                .from(line_items::Column::OrderId)
                .to(orders::Column::Id)
                .into(),
        )
        .select_only()
        .all(&app.db_conn)
        .await
    {
        Ok(result) => {
            let mut temp = vec![];
            for item in result {
                temp.push(OrderModel {
                    id: item.id,
                    loyalty_member_id: item.loyalty_member_id,
                    order_source: item.order_source,
                    order_status: item.order_status,
                })
            }

            (StatusCode::OK, Json(temp))
        }
        Err(err) => {
            // bail!("err: {}", err)
            (StatusCode::OK, Json(vec![]))
        }
    }
}

async fn place_order_handler(
    State(app): State<AppState>,
    Json(input): Json<PlaceOrder>,
) -> impl IntoResponse {
    let txn = app.db_conn.begin().await.unwrap();

    let result = orders::ActiveModel {
        order_source: Set(input.order_source.unwrap_or(0)),
        loyalty_member_id: Set(input.loyalty_member_id.unwrap_or_default()),
        order_status: Set(1),
        ..Default::default()
    }
    .save(&app.db_conn)
    .await
    .unwrap();

    for barista_item in input.barista_items.unwrap() {
        let _ = line_items::ActiveModel {
            item_type: Set(barista_item.item_type.unwrap_or_default()),
            name: Set(barista_item.item_type.unwrap_or_default().to_string()),
            price: Set(Decimal::from_f32_retain(0.0).unwrap_or_default()),
            item_status: Set(0),
            is_barista_order: Set(true),
            order_id: result.id.clone().into(),
            ..Default::default()
        }
        .save(&app.db_conn)
        .await
        .unwrap();
    }

    // kitchen
    for kitchen_item in input.kitchen_items.unwrap() {
        let _ = line_items::ActiveModel {
            item_type: Set(kitchen_item.item_type.unwrap_or_default()),
            name: Set(kitchen_item.item_type.unwrap_or_default().to_string()),
            price: Set(Decimal::from_f32_retain(0.0).unwrap_or_default()),
            item_status: Set(0),
            is_barista_order: Set(false),
            order_id: result.id.clone().into(),
            ..Default::default()
        }
        .save(&app.db_conn)
        .await
        .unwrap();
    }

    txn.commit().await.unwrap();

    result.id.unwrap().to_string()
}

async fn home_handler() -> impl IntoResponse {
    StatusCode::OK
}
