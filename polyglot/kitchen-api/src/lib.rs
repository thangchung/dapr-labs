use anyhow::Result;
use bytes::Bytes;
use cloudevents::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use spin_sdk::{
    http::{Params, Request, Response},
    http_component,
};
use uuid::Uuid;

const DAPR_URL_ENV: &str = "DAPR_URL";

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SubscribeModel {
    pubsubname: String,
    topic: String,
    route: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KitchenOrderPlaced {
    pub order_id: Uuid,
    pub item_lines: Vec<OrderItemDto>,
}

impl TryFrom<&Option<Bytes>> for KitchenOrderPlaced {
    type Error = anyhow::Error;

    fn try_from(value: &Option<Bytes>) -> std::result::Result<Self, Self::Error> {
        match value {
            Some(b) => {
                let cloud_event = serde_json::from_slice::<Event>(b)?;
                let event = match cloud_event.data() {
                    Some(cloudevents::Data::Json(value)) => {
                        let temp = <KitchenOrderPlaced as Deserialize>::deserialize(value);
                        println!("KitchenOrderPlaced event deserialized: {:?}", temp);
                        temp.unwrap()
                    }
                    _ => unreachable!(),
                };
                return Ok(event);
            }
            None => Err(anyhow::anyhow!("No body")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OrderItemDto {
    pub item_line_id: Uuid,
    pub item_type: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct KitchenOrderUpdated {
    pub order_id: Uuid,
    pub item_lines: Vec<OrderItemDto>,
}

/// A simple Spin HTTP component.
#[http_component]
fn handle_kitchen_api(req: Request) -> Result<Response> {
    println!("{:?}", req.headers());
    let mut router = spin_sdk::http::Router::default();
    router.get("/", health_handler);
    router.post("/dapr_subscribe_KitchenOrdered", post_place_order_handler);
    router.handle(req)
}

fn health_handler(_req: Request, _params: Params) -> Result<Response> {
    Ok(http::Response::builder()
        .status(200)
        .body(Some("".into()))?)
}

fn post_place_order_handler(req: Request, _params: Params) -> Result<Response> {
    let dapr_url = std::env::var(DAPR_URL_ENV)?;

    let Ok(model) = KitchenOrderPlaced::try_from(&req.body().clone()) else {
        return Ok(http::Response::builder()
        .status(http::StatusCode::BAD_REQUEST)
        .body(None)?);
    };

    println!("KitchenOrderPlaced event: {:?}", model);

    let mut temp: Vec<OrderItemDto> = vec![];
    for item in model.item_lines {
        //todo: save in dapr state
        // ...

        // copy into another vector
        temp.push(OrderItemDto {
            item_line_id: item.item_line_id,
            item_type: item.item_type,
        })
    }

    pub_order_updated(
        dapr_url.as_str(),
        "kitchenpubsub",
        "kitchenorderupdated",
        KitchenOrderUpdated {
            order_id: model.order_id,
            item_lines: temp,
        },
    );

    Ok(http::Response::builder()
        .status(200)
        .body(Some("".into()))?)
}

fn pub_order_updated(dapr_url: &str, pubsub_name: &str, topic: &str, e: KitchenOrderUpdated) {
    let url = format!("{}/v1.0/publish/{}/{}", dapr_url, pubsub_name, topic);
    println!("url: {}", url);

    let body = bytes::Bytes::from(json!(e).to_string());
    _ = spin_sdk::outbound_http::send_request(
        http::Request::builder()
            .header("Content-Type", "application/json")
            .method("POST")
            .uri(url)
            .body(Some(body))
            .unwrap(),
    )
    .unwrap();
}
