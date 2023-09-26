use anyhow::Result;
use spin_sdk::{
    http::{Request, Response},
    http_component,
    inbound_http::InboundHttp,
};

cargo_component_bindings::generate!();

use bindings::docs::calculator::calculate;

use crate::bindings::docs::calculator::calculate::Op;


struct Component;

// impl InboundHttp for Component {
//     fn handle_request(req: Request) -> Response {

//     }
// }

/// A simple Spin HTTP component.
#[http_component]
fn handle_spin_app(req: Request) -> Result<Response> {
    println!("{:?}", req.headers());

    let res = calculate::eval_expression(Op::Add, 1, 2);

    Ok(http::Response::builder()
        .status(200)
        .header("foo", "bar")
        .body(Some(res.to_string().into()))?)
}

bindings::export!(Component);
