use anyhow::Result;
use serde_json::json;
use spin_sdk::{
    http::{Request, Response},
    http_component,
};

/// A simple Spin HTTP component.
#[http_component]
fn handle_counter_api(req: Request) -> Result<Response> {
    let body = bytes::Bytes::from(json!({"types": "1,2,3"}).to_string());
    let res = spin_sdk::outbound_http::send_request(
        http::Request::builder()
            .method("GET")
            .uri("http://localhost:45983/v1.0/invoke/productapi/method/v1-get-items-by-types")
            .body(Some(body))?,
    )?;
    println!("{:?}", res);

    println!("{:?}", req.headers());
    Ok(http::Response::builder()
        .status(200)
        .header("foo", "bar")
        .body(Some("Hello, Fermyon".into()))?)
}
