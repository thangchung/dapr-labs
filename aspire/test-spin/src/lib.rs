use anyhow::{Context, Result};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::json;
use spin_sdk::http::{
    IntoResponse, Method, Params, Request, RequestBuilder, Response, Router,
};
use spin_sdk::http_component;
use uuid::Uuid;

const DAPR_URL_ENV: &str = "dapr_url";
// const PUB_SUB_NAME: &str = "pubsub";

// #[derive(Debug, Serialize, Clone)]
// #[serde(rename_all = "camelCase")]
// struct SubscribeModel {
//     pubsubname: String,
//     topic: String,
//     route: String,
// }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Pinged {
    pub id: Uuid,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Ponged {
    pub id: Uuid,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ItemType {
    name: String,
    item_type: i8,
    price: f32,
    image: String,
}

#[derive(Debug, Deserialize)]
struct GetItemByTypeModel {  }

impl TryFrom<&Option<Bytes>> for GetItemByTypeModel {
    type Error = anyhow::Error;

    fn try_from(value: &Option<Bytes>) -> std::result::Result<Self, Self::Error> {
        match value {
            Some(b) => Ok(serde_json::from_slice::<GetItemByTypeModel>(b)?),
            None => Err(anyhow::anyhow!("No body")),
        }
    }
}

impl TryFrom<&[u8]> for Pinged {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        serde_json::from_slice::<Pinged>(value)
            .with_context(|| "Could not deserialize value into Pinged model")
    }
}

/// A simple Spin HTTP component.
#[http_component]
fn handle_test_spin(req: Request) -> anyhow::Result<impl IntoResponse> {
    println!("Handling request to {:?}", req.header("spin-full-url"));
    let mut router = Router::default();
    router.get("/", get_home_handler);
    router.get("/v1-get-item-types", get_item_types_handler);
    router.post_async("/pinged", post_ping_handler);
    router.get("/dapr/subscribe", get_dapr_subscribe_handler);
    Ok(router.handle(req))
}

fn get_home_handler(_: Request, _: Params) -> Result<impl IntoResponse> {
    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body("Hello, Fermyon")
        .build())
}

fn get_item_types_handler(_: Request, _: Params) -> Result<impl IntoResponse> {
    let items = json!(get_item_types());
    let result = bytes::Bytes::from(items.to_string());
    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(Some(result))
        .build())
}

fn get_item_types() -> Vec<ItemType> {
    vec![
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
        }
    ]
}

fn get_dapr_subscribe_handler(_: Request, _params: Params) -> Result<impl IntoResponse> {
    let model = json!([
        {
            "pubsubname": "pubsub",
            "topic": "pinged",
            "routes": {
              "rules": [
                {
                  "match": "event.type == 'pinged'",
                  "path": "/pinged"
                },
              ],
              "default": "/pinged"
            }
        }
    ]);

    let result = bytes::Bytes::from(model.to_string());

    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(Some(result))
        .build())
}

async fn post_ping_handler(req: Request, _params: Params) -> Result<impl IntoResponse> {
    let dapr_url = std::env::var(DAPR_URL_ENV)?;

    let Ok(model) = Pinged::try_from(req.body()) else {
        return Ok(Response::builder()
            .status(400)
            .body(Some("Something wrong."))
            .build());
    };

    println!("Pinged event: {:?}", model);

    pub_ponged(
        dapr_url.as_str(),
        "pubsub",
        "ponged",
        Ponged { id: model.id },
    ).await;

    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(Some(""))
        .build())
}

async fn pub_ponged(dapr_url: &str, pubsub_name: &str, topic: &str, e: Ponged) {
    let url = format!("{}/v1.0/publish/{}/{}", dapr_url, pubsub_name, topic);
    println!("url: {}", url);
    println!("Ponged event: {:?}", e);

    let body = bytes::Bytes::from(json!(e).to_string());
    let result = spin_sdk::http::send::<_, Response>(
        RequestBuilder::new(Method::Post, url)
            .header("content-type", "application/json")
            .body(Some(body))
            .build(),
    );

    println!("result: {:?}", result.await.unwrap());
}
