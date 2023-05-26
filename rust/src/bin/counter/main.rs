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
use counter_entity::{line_items, orders, orders::Entity as Order};
use sea_orm::{
    prelude::Decimal, ActiveModelTrait, Database, DatabaseConnection, EntityTrait, ModelTrait, Set,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
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
    #[clap(default_value = "5002", env)]
    app_port: u16,
    #[clap(default_value = "postgres://localhost/db", env)]
    database_url: String,
    #[clap(default_value = "http://localhost:3500", env)]
    dapr_url: String,
    #[clap(default_value = "productapi", env)]
    dapr_product_app: String,
}

#[derive(Clone)]
struct AppState {
    config: Config,
    db_conn: DatabaseConnection,
}

// Command, Query and Models
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlaceOrderItem {
    item_type: Option<i32>,
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
    pub order_lines: Vec<OrderLineModel>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct VecOrderLineModel(Vec<OrderLineModel>);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OrderLineModel {
    pub id: Uuid,
    pub item_type: i32,
    pub name: String,
    pub price: Decimal,
    pub item_status: i32,
    pub is_barista_order: bool,
    pub order_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BaristaOrderIn {
    pub order_id: Uuid,
    pub item_line_id: Uuid,
    pub item_type: i32,
    #[serde(with = "ts_seconds")]
    pub time_in: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct KitchenOrderIn {
    pub order_id: Uuid,
    pub item_line_id: Uuid,
    pub item_type: i32,
    #[serde(with = "ts_seconds")]
    pub time_in: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BaristaOrderUp {
    pub order_id: Uuid,
    pub item_line_id: Uuid,
    pub name: String,
    pub item_type: i32,
    #[serde(deserialize_with = "from_ts")]
    pub time_in: DateTime<Utc>,
    pub made_by: String,
    #[serde(deserialize_with = "from_ts")]
    pub time_up: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KitchenOrderUp {
    pub order_id: Uuid,
    pub item_line_id: Uuid,
    pub name: String,
    pub item_type: i32,
    #[serde(deserialize_with = "from_ts")]
    pub time_in: DateTime<Utc>,
    pub made_by: String,
    #[serde(deserialize_with = "from_ts")]
    pub time_up: DateTime<Utc>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ItemTypeDto {
    price: f32,
    item_type: i32,
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
        .route(
            "/update-barista-order-line-item",
            post(update_barista_order_line_item_handler),
        )
        .route(
            "/update-kitchen-order-line-item",
            post(update_kitchen_order_line_item_handler),
        )
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

async fn get_order_handler(State(app): State<AppState>) -> impl IntoResponse {
    let ord = Order::find().all(&app.db_conn).await;

    match ord {
        Ok(result) => {
            let mut temp = vec![];
            for order in result {
                let mut order_model = OrderModel {
                    id: order.id,
                    loyalty_member_id: order.loyalty_member_id,
                    order_source: order.order_source,
                    order_status: order.order_status,
                    order_lines: Vec::new(),
                };

                let line_items = order
                    .find_related(line_items::Entity)
                    .all(&app.db_conn)
                    .await
                    .unwrap_or_default();
                for line_item in line_items {
                    order_model.order_lines.push(OrderLineModel {
                        id: line_item.id,
                        is_barista_order: line_item.is_barista_order,
                        item_status: line_item.item_status,
                        item_type: line_item.item_type,
                        name: line_item.name,
                        order_id: line_item.order_id,
                        price: line_item.price,
                    })
                }

                temp.push(order_model);
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

    // barista
    let barista_items_vec = input.barista_items.unwrap();
    if barista_items_vec.iter().len() > 0 {
        let params = process_params(&barista_items_vec);
        let product_items =
            get_product_items(&app.config.dapr_url, &app.config.dapr_product_app, params).await;
        tracing::debug!("product_items: {:?}", product_items);

        for barista_item in barista_items_vec {
            let product_item_result = product_items
                .clone()
                .into_iter()
                .find(|i| i.item_type == barista_item.item_type.unwrap_or_default());

            let price = if let Some(product_item) = product_item_result {
                product_item.price
            } else {
                0.0
            };

            let order_line_result = line_items::ActiveModel {
                item_type: Set(barista_item.item_type.unwrap_or_default()),
                name: Set(barista_item.item_type.unwrap_or_default().to_string()),
                price: Set(Decimal::from_f32_retain(price).unwrap_or_default()),
                item_status: Set(0),
                is_barista_order: Set(true),
                order_id: result.id.clone().into(),
                ..Default::default()
            }
            .save(&app.db_conn)
            .await
            .unwrap();

            // publish domain event
            publish_barista_order_in_event(
                &app.config.dapr_url,
                "baristapubsub",
                "baristaordered",
                BaristaOrderIn {
                    order_id: result.id.clone().take().unwrap(),
                    item_line_id: order_line_result.id.clone().take().unwrap(),
                    item_type: barista_item.item_type.unwrap_or_default(),
                    time_in: Utc::now(),
                },
            )
            .await;
        }
    }

    // kitchen
    let kitchen_items_vec = input.kitchen_items.unwrap();
    if kitchen_items_vec.iter().len() > 0 {
        let params = process_params(&kitchen_items_vec);
        let product_items =
            get_product_items(&app.config.dapr_url, &app.config.dapr_product_app, params).await;
        tracing::debug!("product_items: {:?}", product_items);

        for kitchen_item in kitchen_items_vec {
            let product_item_result = product_items
                .clone()
                .into_iter()
                .find(|i| i.item_type == kitchen_item.item_type.unwrap_or_default());

            let price = if let Some(product_item) = product_item_result {
                product_item.price
            } else {
                0.0
            };

            let order_line_result = line_items::ActiveModel {
                item_type: Set(kitchen_item.item_type.unwrap_or_default()),
                name: Set(kitchen_item.item_type.unwrap_or_default().to_string()),
                price: Set(Decimal::from_f32_retain(price).unwrap_or_default()),
                item_status: Set(0),
                is_barista_order: Set(false),
                order_id: result.id.clone().into(),
                ..Default::default()
            }
            .save(&app.db_conn)
            .await
            .unwrap();

            // publish domain event
            publish_kitchen_order_in_event(
                &app.config.dapr_url,
                "kitchenpubsub",
                "kitchenordered",
                KitchenOrderIn {
                    order_id: result.id.clone().take().unwrap(),
                    item_line_id: order_line_result.id.clone().take().unwrap(),
                    item_type: kitchen_item.item_type.unwrap_or_default(),
                    time_in: Utc::now(),
                },
            )
            .await;
        }
    }

    txn.commit().await.unwrap();

    result.id.unwrap().to_string()
}

async fn home_handler() -> impl IntoResponse {
    StatusCode::OK
}

async fn get_subscribe_handler() -> impl IntoResponse {
    let subscribe_model = vec![
        SubscribeModel {
            pubsubname: "baristaorderuppubsub".to_string(),
            topic: "baristaorderup".to_string(),
            route: "update-barista-order-line-item".to_string(),
        },
        SubscribeModel {
            pubsubname: "kitchenorderuppubsub".to_string(),
            topic: "kitchenorderup".to_string(),
            route: "update-kitchen-order-line-item".to_string(),
        },
    ];

    (StatusCode::OK, Json(subscribe_model))
}

async fn update_barista_order_line_item_handler(
    State(app): State<AppState>,
    Json(event): Json<Event>,
) -> impl IntoResponse {
    tracing::debug!("barista_order_up_event: {:?}", event.data());

    let event = match event.data() {
        Some(cloudevents::Data::Json(value)) => {
            let temp = <BaristaOrderUp as Deserialize>::deserialize(value);
            tracing::debug!("BaristaOrderUp: {:?}", temp);
            temp.unwrap()
        }
        _ => unreachable!(),
    };

    match Order::find_by_id(event.order_id).one(&app.db_conn).await {
        Ok(result) => {
            tracing::debug!("Order_updating: {:?}", result);
            if let Some(order) = result {
                let line_item_result = order
                    .find_related(line_items::Entity)
                    .one(&app.db_conn)
                    .await
                    .unwrap_or_default();

                if let Some(line_item) = line_item_result {
                    if line_item.is_barista_order {
                        let _ = line_items::ActiveModel {
                            id: Set(line_item.id),
                            item_status: Set(2), // 0=PLACED; 1=IN_PROGRESS; 2=FULFILLED
                            ..Default::default()
                        }
                        .save(&app.db_conn)
                        .await
                        .unwrap();
                    }
                }

                let line_item_all_result = order
                    .find_related(line_items::Entity)
                    .all(&app.db_conn)
                    .await
                    .unwrap_or_default();

                let mut all_done = true; // assume all done
                for line_item in line_item_all_result {
                    if line_item.item_status < 2 {
                        all_done = false;
                    }
                }

                if all_done {
                    let _ = orders::ActiveModel {
                        id: Set(order.id),
                        order_status: Set(2), // 0=PLACED; 1=IN_PROGRESS; 2=FULFILLED
                        ..Default::default()
                    }
                    .save(&app.db_conn)
                    .await
                    .unwrap();
                }
            };
        }
        Err(_) => unreachable!(),
    };
}

async fn update_kitchen_order_line_item_handler(
    State(app): State<AppState>,
    Json(event): Json<Event>,
) -> impl IntoResponse {
    tracing::debug!("kitchen_order_up_event: {:?}", event.data());

    let event = match event.data() {
        Some(cloudevents::Data::Json(value)) => {
            let temp = <KitchenOrderUp as Deserialize>::deserialize(value);
            tracing::debug!("KitchenOrderUp: {:?}", temp);
            temp.unwrap()
        }
        _ => unreachable!(),
    };

    match Order::find_by_id(event.order_id).one(&app.db_conn).await {
        Ok(result) => {
            tracing::debug!("Order_updating: {:?}", result);
            if let Some(order) = result {
                let line_item_result = order
                    .find_related(line_items::Entity)
                    .one(&app.db_conn)
                    .await
                    .unwrap_or_default();

                if let Some(line_item) = line_item_result {
                    // kitchen item?
                    if !line_item.is_barista_order {
                        let _ = line_items::ActiveModel {
                            id: Set(line_item.id),
                            item_status: Set(2), // 0=PLACED; 1=IN_PROGRESS; 2=FULFILLED
                            ..Default::default()
                        }
                        .save(&app.db_conn)
                        .await
                        .unwrap();
                    }
                }

                let line_item_all_result = order
                    .find_related(line_items::Entity)
                    .all(&app.db_conn)
                    .await
                    .unwrap_or_default();

                let mut all_done = true; // assume all done
                for line_item in line_item_all_result {
                    if line_item.item_status < 2 {
                        all_done = false;
                    }
                }

                if all_done {
                    let _ = orders::ActiveModel {
                        id: Set(order.id),
                        order_status: Set(2), // 0=PLACED; 1=IN_PROGRESS; 2=FULFILLED
                        ..Default::default()
                    }
                    .save(&app.db_conn)
                    .await
                    .unwrap();
                }
            };
        }
        Err(_) => unreachable!(),
    };
}

fn process_params(items_vec: &[PlaceOrderItem]) -> String {
    let params = items_vec.iter().fold("".to_string(), |acc, x| {
        if let Some(item_type) = x.item_type {
            tracing::debug!("item_type: {:?}", x);
            format!("{acc},{}", item_type)
        } else {
            "".to_string()
        }
    });

    params
}

async fn get_product_items(
    dapr_url: &str,
    dapr_product_app: &str,
    params: String,
) -> Vec<ItemTypeDto> {
    let url = format!(
        "{}/v1.0/invoke/{}/method/v1-get-items-by-types",
        dapr_url, dapr_product_app,
    );
    tracing::debug!("url: {}", url);

    surf::get(url)
        .body(json!({ "types": params.trim_start_matches(',')}))
        .recv_json::<Vec<ItemTypeDto>>()
        .await
        .unwrap()
}

async fn publish_barista_order_in_event(
    dapr_url: &str,
    pubsub_name: &str,
    topic: &str,
    event: BaristaOrderIn,
) {
    let url = format!("{}/v1.0/publish/{}/{}", dapr_url, pubsub_name, topic);
    tracing::debug!("url: {}", url);

    surf::post(url).body(json!(event)).await.unwrap();
}

async fn publish_kitchen_order_in_event(
    dapr_url: &str,
    pubsub_name: &str,
    topic: &str,
    event: KitchenOrderIn,
) {
    let url = format!("{}/v1.0/publish/{}/{}", dapr_url, pubsub_name, topic);
    tracing::debug!("url: {}", url);

    surf::post(url).body(json!(event)).await.unwrap();
}
