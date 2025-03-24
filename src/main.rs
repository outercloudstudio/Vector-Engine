mod clips;
mod renderer;
mod runtime;

use std::{
    env,
    fs::File,
    io::{BufWriter, Write},
    thread,
    time::Instant,
};

use ash::vk;
use cgmath::{vec2, vec4};
use clips::ScriptClip;
use ffmpeg_sidecar::{
    command::FfmpegCommand,
    event::{FfmpegEvent, LogLevel},
};
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

    ffmpeg_sidecar::download::auto_download().unwrap();

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

    let now = Instant::now();

    let render_target = clip.render_elements(&vec![Elements::Rect(rect)], &renderer, 1920, 1080, renderer::renderer::RenderMode::Raw);

    let bytes = render_target.to_raw(&renderer);

    println!("Render to target at {}ms", now.elapsed().as_millis());

    let mut output = FfmpegCommand::new()
        .args(["-f", "rawvideo", "-pix_fmt", "rgba", "-s", "1920x1080", "-r", "30"])
        .input("-")
        .args(["-c:v", "libx265", "-pix_fmt", "yuva420p"])
        .args(["-y", "renders/render.mp4"])
        .spawn()
        .unwrap();

    let mut stdin = output.take_stdin().unwrap();
    thread::spawn(move || {
        for _ in 0..60 {
            stdin.write_all(&bytes).ok();
        }
    });

    output.iter().unwrap().for_each(|e| match e {
        FfmpegEvent::Log(LogLevel::Error, e) => println!("Error: {}", e),
        FfmpegEvent::Progress(p) => println!("Progress: {} / 00:00:15", p.time),
        _ => {}
    });

    println!("Render fully at {}ms", now.elapsed().as_millis());
}
