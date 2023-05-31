use anyhow::Result;
use rust_decimal::prelude::*;
use serde::Serialize;
use serde_json::json;
use spin_sdk::{
    http::{Request, Response},
    http_component,
    pg::{self, Decode},
};
use uuid::Uuid;

const DB_URL_ENV: &str = "DB_URL";

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

    fn pop(vec: &[u8]) -> &[u8; 16] {
        vec.try_into().expect("slice with incorrect length")
    }
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

    get_orders()

    // println!("{:?}", req.headers());
    // Ok(http::Response::builder()
    //     .status(200)
    //     .body(Some("Hello, Fermyon".into()))?)
}

fn get_orders() -> Result<Response> {
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
