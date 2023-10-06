cargo_component_bindings::generate!();

use bindings::exports::docs::calculator::add::Guest;

struct Component;

impl Guest for Component {
    fn add(a: i32, b: i32) -> i32 {
        a + b
    }
}
