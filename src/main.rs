mod clips;
mod renderer;
mod runtime;

use image::ImageEncoder;
use log::info;
use notify::{Event, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::{env, fs::File, io::BufWriter};

use clips::{ClipLoader, Clips};
use renderer::renderer::Renderer;

struct Timeline {}

pub enum Command {
    Preview(u32, Sender<Vec<u8>>),
    Render(u32),
    PlaygroundUpdate,
}

fn main() {
    env::set_var("RUST_LOG", "info");
    env::set_var("RUST_BACKTRACE", "1");

    pretty_env_logger::init();

    let renderer = Renderer::new();
}
