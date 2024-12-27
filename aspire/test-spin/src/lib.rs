use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::json;
use spin_sdk::http::{IntoResponse, Params, Request, Response, Router};
use spin_sdk::http_component;
use anyhow::{Result};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ItemType {
    name: String,
    item_type: i8,
    price: f32,
    image: String,
}

#[derive(Debug, Deserialize)]
struct GetItemByTypeModel{
    types: String,
}

impl TryFrom<&Option<Bytes>> for GetItemByTypeModel {
    type Error = anyhow::Error;

    fn try_from(value: &Option<Bytes>) -> std::result::Result<Self, Self::Error> {
        match value {
            Some(b) => Ok(serde_json::from_slice::<GetItemByTypeModel>(b)?),
            None => Err(anyhow::anyhow!("No body")),
        }
    }
}

/// A simple Spin HTTP component.
#[http_component]
fn handle_test_spin(req: Request) -> anyhow::Result<impl IntoResponse> {
    println!("Handling request to {:?}", req.header("spin-full-url"));
    let mut router = Router::default();
    router.get("/", get_home_handler);
    router.get("/v1-get-item-types", get_item_types_handler);
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
    ]
}
