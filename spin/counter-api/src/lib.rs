use anyhow::Result;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use spin_sdk::{
    http::{Params, Request, Response},
    http_component,
    pg::{self, Decode, ParameterValue},
};
use uuid::Uuid;

const DB_URL_ENV: &str = "DB_URL";
const DAPR_URL_ENV: &str = "DAPR_URL";
const PRODUCT_APP_ENV: &str = "PRODUCT_APP";

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct PlaceOrderItem {
    item_type: Option<i32>,
}

#[derive(Debug, Clone)]
struct PlaceOrderItemVec(Vec<PlaceOrderItem>);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlaceOrder {
    command_type: Option<i32>,
    order_source: Option<i32>,
    location: Option<i32>,
    loyalty_member_id: Uuid,
    barista_items: Option<Vec<PlaceOrderItem>>,
    kitchen_items: Option<Vec<PlaceOrderItem>>,
    timestamp: Option<DateTime<Utc>>,
}

impl TryFrom<&Option<Bytes>> for PlaceOrder {
    type Error = anyhow::Error;

    fn try_from(value: &Option<Bytes>) -> std::result::Result<Self, Self::Error> {
        match value {
            Some(b) => Ok(serde_json::from_slice::<PlaceOrder>(b)?),
            None => Err(anyhow::anyhow!("No body")),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OrderModel {
    pub id: String,
    pub order_source: i32,
    pub loyalty_member_id: Option<Uuid>,
    pub order_status: i32,
    pub order_lines: Option<Vec<OrderLineModel>>,
}

impl OrderModel {
    fn from_row(row: &spin_sdk::pg::Row) -> Result<Self> {
        let order_id = String::decode(&row[0])?;
        let order_source = 0; // from web
        let order_status = i32::decode(&row[3])?;
        Ok(OrderModel {
            id: order_id,
            order_source,
            order_status,
            loyalty_member_id: None,
            order_lines: None,
        })
    }

    // fn pop(vec: &[u8]) -> &[u8; 16] {
    //     vec.try_into().expect("slice with incorrect length")
    // }
}

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

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SubscribeModel {
    pubsubname: String,
    topic: String,
    route: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ItemTypeDto {
    price: f32,
    item_type: i32,
}

/// A simple Spin HTTP component.
#[http_component]
fn handle_counter_api(req: Request) -> Result<Response> {
    // let body = bytes::Bytes::from(json!({"types": "1,2,3"}).to_string());
    // let res = spin_sdk::outbound_http::send_request(
    //     http::Request::builder()
    //         .method("GET")
    //         .uri("http://localhost:45983/v1.0/invoke/productapi/method/v1-get-items-by-types")
    //         .body(Some(body))?,
    // )?;
    // println!("{:?}", res);

    println!("{:?}", req.headers());
    let mut router = spin_sdk::http::Router::default();
    router.get("/", health_handler);
    router.get("/v1/api/fulfillment-orders", get_orders_handler);
    router.post("/v1/api/orders", place_order_handler);
    router.handle(req)
}

fn health_handler(_req: Request, _params: Params) -> Result<Response> {
    Ok(http::Response::builder()
        .status(200)
        .body(Some("".into()))?)
}

fn get_orders_handler(_req: Request, _params: Params) -> Result<Response> {
    let address = std::env::var(DB_URL_ENV)?;

    // let order_id = uuid::Uuid::new_v4().clone();
    // let order_id_param = ParameterValue::Binary(order_id.as_ref());
    // let params = vec![order_id_param];
    // let sql = "
    //     SELECT l.id, l.created, l.is_barista_order, l.item_status, l.item_type, l.name, l.order_id, l.price, l.updated, t.id
    //     FROM (
    //         SELECT o.id
    //         FROM \"order\".orders AS o
    //         WHERE o.id = $1
    //         LIMIT 1
    //     ) AS t
    //     INNER JOIN \"order\".line_items AS l ON t.id = l.order_id
    //     ORDER BY t.id";

    let params = vec![];
    let sql = "
        SELECT l.id::text, l.created, l.is_barista_order, l.item_status, l.item_type, l.name, l.order_id, l.price, l.updated, t.id
        FROM (
            SELECT o.id
            FROM \"order\".orders AS o
            LIMIT 1
        ) AS t
        INNER JOIN \"order\".line_items AS l ON t.id = l.order_id
        ORDER BY t.id";

    let mut order_list: Vec<OrderModel> = vec![];
    let rowset = pg::query(&address, sql, &params)?;
    for row in rowset.rows {
        let Ok(order) = OrderModel::from_row(&row) else {
            return Err(anyhow::anyhow!("Error!"));
        };
        order_list.push(order);
    }

    let order_list_json = json!(order_list);
    let result = bytes::Bytes::from(order_list_json.to_string());

    Ok(http::Response::builder()
        .header("Content-Type", "application/json")
        .status(200)
        .body(Some(result))?)
}

fn place_order_handler(req: Request, _params: Params) -> Result<Response> {
    let address = std::env::var(DB_URL_ENV)?;
    let dapr_url = std::env::var(DAPR_URL_ENV)?;
    let product_app = std::env::var(PRODUCT_APP_ENV)?;

    let Ok(model) = PlaceOrder::try_from(&req.body().clone()) else {
        return Ok(http::Response::builder()
        .status(http::StatusCode::BAD_REQUEST)
        .body(None)?);
    };

    println!("Model: {:?}", model);

    let query_sql = "SELECT currval(pg_get_serial_sequence('\"order\".\"orders\"', 'id'))::text";
    let insert_order_sql =
        "INSERT INTO \"order\".orders (id, loyalty_member_id, order_source, order_status)
        VALUES ($1, $2, $3, $4)
        RETURNING id";
    let insert_order_item_sql = "INSERT INTO \"order\".line_items (is_barista_order, item_status, item_type, name, order_id, price)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id";

    let loyalty_member_id = model.loyalty_member_id.to_string();
    // .unwrap_or(uuid::Uuid::new_v4().clone());

    // let loyalty_member_id = model.loyalty_member_id.unwrap().to_string();
    // let aaa = loyalty_member_id.as_ref();
    // let loyalty_member_id_param = ParameterValue::Str(aaa);

    let order_id = uuid::Uuid::new_v4().clone();
    let order_id_param = ParameterValue::Binary(order_id.as_ref());
    let loyalty_member_id_param = ParameterValue::Str(&loyalty_member_id);
    let insert_order_params = vec![
        order_id_param,
        loyalty_member_id_param,
        ParameterValue::Int32(model.order_source.unwrap_or_default()),
        ParameterValue::Int32(0), // processing
    ];

    let order_nrow_executed = pg::execute(&address, insert_order_sql, &insert_order_params)?;
    println!("order_nrow_executed: {}", order_nrow_executed);
    
    let barista_items = match model.barista_items {
        Some(barista_items) => barista_items,
        None => {
            let temp: Vec<PlaceOrderItem> = vec![];
            temp
        }
    };

    if barista_items.clone().into_iter().len() > 0 {
        let aaa = barista_items.to_vec();
        let params = process_params(aaa);
        let product_items = get_product_items(&dapr_url, &product_app, params);
        println!("product_items: {:?}", product_items);

        let rowset = spin_sdk::pg::query(&address, query_sql, &[])?;
        match rowset.rows.first() {
            Some(row) => {
                let new_id = String::decode(&row[0])?;
                for order_item in barista_items {
                    let product_item_result = product_items
                        .as_ref()
                        .unwrap()
                        .into_iter()
                        .find(|i| i.item_type == order_item.item_type.unwrap_or_default());

                    let price = if let Some(product_item) = product_item_result {
                        product_item.price
                    } else {
                        0.0
                    };

                    let item_type_param =
                        ParameterValue::Int32(order_item.item_type.unwrap_or_default());
                    let item_type_name_param = ParameterValue::Str("name"); //todo: hardcode
                    let order_id_param = ParameterValue::Binary(new_id.as_ref());
                    let price_param = ParameterValue::Floating32(price);
                    let insert_order_item_params = vec![
                        ParameterValue::Boolean(true),
                        ParameterValue::Int32(0), // processing
                        item_type_param,
                        item_type_name_param,
                        order_id_param,
                        price_param,
                    ];

                    let barista_order_item_nrow_executed =
                        pg::execute(&address, insert_order_item_sql, &insert_order_item_params)?;
                    println!(
                        "barista_order_item_nrow_executed: {}",
                        barista_order_item_nrow_executed
                    );
                }

                return Ok(http::Response::builder()
                    .status(200)
                    .body(Some("".into()))?);
            }
            None => {
                return Ok(http::Response::builder()
                    .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                    .header(http::header::CONTENT_TYPE, "text/plain")
                    .body(Some("Could not persist product".into()))?)
            }
        };
    }

    Ok(http::Response::builder()
        .status(200)
        .body(Some("".into()))?)
}

fn process_params(items_vec: Vec<PlaceOrderItem>) -> String {
    let params = items_vec.iter().fold("".to_string(), |acc, x| {
        if let Some(item_type) = x.item_type {
            println!("item_type: {:?}", x);
            format!("{acc},{}", item_type)
        } else {
            "".to_string()
        }
    });

    params
}

fn get_product_items(
    dapr_url: &str,
    dapr_product_app: &str,
    params: String,
) -> Result<Vec<ItemTypeDto>> {
    println!("run get_product_items");
    let body = bytes::Bytes::from(json!({ "types": params }).to_string());
    let url = format!(
        "{}/v1.0/invoke/{}/method/v1-get-items-by-types",
        dapr_url, dapr_product_app,
    );
    let res = spin_sdk::outbound_http::send_request(
        http::Request::builder()
            .method("GET")
            .uri(url)
            .body(Some(body))?,
    )?;

    let Some(res_body) = res.body() else {
        return Ok(vec![]);
    };

    let result = serde_json::from_slice::<Vec<ItemTypeDto>>(res_body)?;

    Ok(result)
}
