mod clips;
mod renderer;
mod runtime;

use std::{env, fs::File, io::BufWriter};

use ash::vk;
use cgmath::{vec2, vec4};
use clips::ScriptClip;
use image::ImageEncoder;
use log::info;

use renderer::{
    elements::{Elements, Rect},
    renderer::{RenderTarget, Renderer},
};

fn main() {
    env::set_var("RUST_LOG", "info");
    env::set_var("RUST_BACKTRACE", "1");

    pretty_env_logger::init();

    let renderer = Renderer::new();

    let rect = Rect {
        position: vec2(0f32, 0f32),
        origin: vec2(0.5f32, 0.5f32),
        color: vec4(1f32, 1f32, 1f32, 1f32),
        order: 0f32,
        radius: 0f32,
        rotation: 0f32,
        size: vec2(300f32, 300f32),
    };

    let clip = ScriptClip::new(String::from("console.log()"), &renderer);
    let render_target = clip.render_elements(&vec![Elements::Rect(rect)], &renderer, 1920, 1080, renderer::renderer::RenderMode::Raw);

    let bytes = render_target.to_raw(&renderer);

    let file = File::create("./renders/render.png").unwrap();
    let mut file_writer = BufWriter::new(file);

    let encoder = image::codecs::png::PngEncoder::new(&mut file_writer);
    encoder.write_image(&bytes, 1920, 1080, image::ColorType::Rgba8).unwrap();
}
