cargo_component_bindings::generate!();

use bindings::{docs::calculator::add::add, exports::docs::calculator::calculate::Op};

use crate::bindings::exports::docs::calculator::calculate::Guest;

struct Component;

impl Guest for Component {
    fn eval_expression(op: Op, x: i32, y: i32) -> i32 {
        match op {
            Op::Add => add(x, y),
        }
    }
}
