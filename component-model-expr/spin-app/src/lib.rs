wit_bindgen::generate!("http-trigger" in "../wit/deps/spin");

// mod calculator;

use bindings::calculator_calculate::{Op, eval_expression};
// use bindings::docs::calculator::calculate::{eval_expression, Op};
// use bindings::exports::docs::calculator::calculate::Guest;
use exports::fermyon::spin::inbound_http::{self, Request, Response};
use serde::Deserialize;
use anyhow::{anyhow, Result};

cargo_component_bindings::generate!();

struct SpinHttp;
export_http_trigger!(SpinHttp);

impl inbound_http::InboundHttp for SpinHttp {
    fn handle_request(req: Request) -> Response {
        calculate(Self, req).unwrap_or_else(|e| {
            Response {
                status: 500,
                headers: None,
                body: Some(format!("Error: {}", e).into_bytes()),
            }
        })
    }
}

fn calculate(s: SpinHttp, req: fermyon::spin::http_types::Request) -> Result<Response> {
    let query = req.uri.split('?').nth(1).ok_or_else(|| anyhow!( "No query string found"))?;
    let params: QueryParams = serde_qs::from_str(query)?;
    let op = match params.op.as_ref() {
        "add" => Op::Add,
        _ => anyhow::bail!("Unknown operation: {}", params.op)
    };

    let result = eval_expression(op, params.x, params.y);

    Ok(Response {
        status: 200,
        headers: None,
        body: Some(format!("Result of operation '{}' with values `{}`,`{}`: {result}", params.op, params.x, params.y).into_bytes()),
    })
}

#[derive(Debug, Deserialize)]
struct QueryParams {
    x: i32,
    y: i32,
    op: String,
}

// #[http_component]
// fn handle_spin_app(req: Request) -> Result<Response> {
//     println!("{:?}", req.headers());

//     let res = calculate::eval_expression(Op::Add, 1, 2);

//     Ok(http::Response::builder()
//         .status(200)
//         .header("foo", "bar")
//         .body(Some(res.to_string().into()))?)
// }

// bindings::export!(Component);
