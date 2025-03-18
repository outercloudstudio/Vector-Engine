mod clips;
mod renderer;
mod runtime;

use std::{env, fs::File, io::BufWriter};

use image::ImageEncoder;
use log::info;

use renderer::renderer::{RenderTarget, Renderer};

fn main() {
    env::set_var("RUST_LOG", "info");
    env::set_var("RUST_BACKTRACE", "1");

    pretty_env_logger::init();

    let renderer = Renderer::new();

    let render_target = RenderTarget::new(3, 3, &renderer, renderer::renderer::RenderMode::Raw);
    let bytes = render_target.to_raw(&renderer);

    log::info!("{:?}", bytes);

    // let file = File::create("./renders/render.png").unwrap();
    // let mut file_writer = BufWriter::new(file);

    // let encoder = image::codecs::png::PngEncoder::new(&mut file_writer);
    // encoder.write_image(&bytes, 5, 5, image::ColorType::Rgba8).unwrap();
}
