cargo_component_bindings::generate!();

use bindings::{docs::calculator::add::add, exports::docs::calculator::calculate::Op};

use crate::bindings::exports::docs::calculator::calculate::Guest;

struct Component;

impl Guest for Component {
    fn eval_expression(op: Op, x: u32, y: u32) -> u32 {
        match op {
            Op::Add => add(x, y),
        }
    }
}
