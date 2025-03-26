mod clips;
mod renderer;
mod runtime;

use std::{env, io::Write, thread, time::Instant};

use cgmath::{vec2, vec4};
use clips::{ClipLoader, ScriptClip};
use ffmpeg_sidecar::{
    command::FfmpegCommand,
    event::{FfmpegEvent, LogLevel},
};
use log::info;

use renderer::{
    elements::{Elements, Rect},
    renderer::Renderer,
};

fn main() {
    env::set_var("RUST_LOG", "info");
    env::set_var("RUST_BACKTRACE", "1");

    pretty_env_logger::init();

    ffmpeg_sidecar::download::auto_download().unwrap();

    let now = Instant::now();

    let mut output = FfmpegCommand::new()
        .args(["-f", "rawvideo", "-pix_fmt", "rgba", "-s", "1920x1080", "-r", "30"])
        .input("-")
        .args(["-c:v", "libx265", "-pix_fmt", "yuva420p"])
        .args(["-y", "renders/render.mp4"])
        .spawn()
        .unwrap();

    let mut stdin = output.take_stdin().unwrap();
    thread::spawn(move || {
        let mut renderer = Renderer::new();
        let mut clip = ScriptClip::new(
            String::from(
                "clip(function* () {
	const rect = add(
		new Rect({
			size: new Vector2(400, 400),
			color: new Vector4(0.5, 0.5, 0, 1),
		})
	)

	yield* rect.color.to(new Vector4(0, 0.5, 0.5, 1), 1, linear)
	yield* rect.color.to(new Vector4(0.5, 0, 0.5, 1), 1, linear)
	yield* rect.color.to(new Vector4(0.5, 0.5, 0, 1), 1, linear)
})",
            ),
            &renderer,
        );
        let mut clip_loader = &mut ClipLoader::new();

        let mut total_frame_time = 0;
        let mut total_frames = 0;

        for i in 0..60 {
            clip.set_frame(i);

            let frame_now = Instant::now();
            let bytes = clip.render_to_raw(&mut renderer, &mut clip_loader, 1920, 1080);
            total_frame_time += frame_now.elapsed().as_millis();
            total_frames += 1;

            stdin.write_all(&bytes).ok();
        }

        info!("Average frame time {}ms", total_frame_time / total_frames)
    });

    output.iter().unwrap().for_each(|e| match e {
        FfmpegEvent::Log(LogLevel::Error, e) => println!("Error: {}", e),
        FfmpegEvent::Progress(p) => println!("Progress: {} / 00:00:15", p.time),
        _ => {}
    });

    println!("Render fully at {}ms", now.elapsed().as_millis());
}
