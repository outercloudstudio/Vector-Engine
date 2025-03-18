mod clips;
mod renderer;
mod runtime;

use std::env;

use log::info;

use renderer::renderer::Renderer;

fn main() {
    env::set_var("RUST_LOG", "info");
    env::set_var("RUST_BACKTRACE", "1");

    pretty_env_logger::init();

    let renderer = Renderer::new();
}
