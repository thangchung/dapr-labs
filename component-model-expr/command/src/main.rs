use std::{thread::sleep, fmt};

use bindings::docs::calculator::calculate::{Op, self};
use clap::Parser;

cargo_component_bindings::generate!();

fn parse_operator(op: &str) -> anyhow::Result<Op> {
    match op {
        "add" => Ok(Op::Add),
        _ => anyhow::bail!("Unknown operation: {}", op),
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::Add => write!(f, "+"),
        }
    }
}

#[derive(Parser)]
#[clap(name = "calculator", version = env!("CARGO_PKG_VERSION"))]
struct Command {
    x: u32,
    y: u32,
    #[clap(value_parser = parse_operator)]
    op: Op,
}

impl Command {
    fn run(self) {
        let res = calculate::eval_expression(self.op, self.x, self.y);
        println!("{} {} {} = {res}", self.x, self.op, self.y);
        sleep(std::time::Duration::from_millis(10))
    }
}

fn main() {
    Command::parse().run()
}
