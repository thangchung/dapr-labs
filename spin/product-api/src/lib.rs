use anyhow::Result;
use bytes::Bytes;
use serde::{Serialize, Deserialize};
use serde_json::json;
use spin_sdk::{
    http::{Params, Request, Response},
    http_component,
};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ItemType {
    name: String,
    item_type: i8,
    price: f32,
    image: String,
}

#[http_component]
fn handle_product_api(req: Request) -> Result<Response> {
    println!("{:?}", req.headers());
    let mut router = spin_sdk::http::Router::default();
    router.get("/", health_handler);
    router.get("/v1-get-item-types", get_item_types_handler);
    router.get("/v1-get-items-by-types", get_item_by_types_handler);
    router.handle(req)
}

fn health_handler(_req: Request, _params: Params) -> Result<Response> {
    Ok(http::Response::builder()
        .status(200)
        .body(Some("".into()))?)
}

fn get_item_types_handler(_req: Request, _params: Params) -> Result<Response> {
    let items = json!(get_item_types());
    let result = bytes::Bytes::from(items.to_string());
    Ok(http::Response::builder()
        .header("Content-Type", "application/json")
        .status(200)
        .body(Some(result))?)
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

fn get_item_by_types_handler(req: Request, _params: Params) -> Result<Response> {
    // let Some(types) = params.get("types") else {
    //     return Ok(http::Response::builder()
    //     .status(http::StatusCode::NOT_FOUND)
    //     .body(None)?);
    // };

    let Ok(model) = GetItemByTypeModel::try_from(&req.body().clone()) else {
        return Ok(http::Response::builder()
        .status(http::StatusCode::BAD_REQUEST)
        .body(None)?);
    };

    let mut temp: Vec<ItemType> = Vec::new();

    for i in get_item_types() {
        let parts = model.types.split(',');

        let ii = i.clone();
        for j in parts {
            if ii.item_type.to_string().as_str() == j {
                temp.push(ii.clone())
            }
        }
    }

    let result = bytes::Bytes::from(json!(temp).to_string());
    Ok(http::Response::builder()
        .header("Content-Type", "application/json")
        .status(200)
        .body(Some(result))?)
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
