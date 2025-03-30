mod clips;
mod renderer;
mod runtime;

use std::{env, io::Write, path::Path, sync::mpsc::channel, thread, time::Instant};

use ash::vk;
use cgmath::{vec2, vec4};
use clips::{ClipLoader, Clips, ScriptClip};
use ffmpeg_sidecar::{
    command::FfmpegCommand,
    event::{FfmpegEvent, LogLevel},
};
use log::info;

use notify::{RecursiveMode, Watcher};
use renderer::{
    elements::{Elements, Rect},
    renderer::Renderer,
};
use winit::{
    application::ApplicationHandler,
    event::{Event, WindowEvent},
};
use winit::{dpi::LogicalSize, platform::run_on_demand::EventLoopExtRunOnDemand};
use winit::{dpi::Size, window::Window};
use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::WindowAttributes,
};

struct Editor {
    renderer: Option<Renderer>,
    window: Option<Window>,
}

impl Editor {
    pub fn new() -> Editor {
        return Editor { renderer: None, window: None };
    }

    pub fn open(mut self) {
        let event_loop = EventLoop::new().unwrap();

        event_loop.run_app(&mut self);

        // let window = WindowBuilder::new().with_title("Vector Engine").with_inner_size(LogicalSize::new(800, 600)).build(&event_loop).unwrap();

        // event_loop.run(move |event, _, control_flow| {
        //     *control_flow = ControlFlow::Poll;

        //     match event {
        //         Event::WindowEvent { event, .. } => match event {
        //             WindowEvent::CloseRequested => {
        //                 *control_flow = ControlFlow::Exit;
        //             }
        //             _ => {}
        //         },
        //         Event::MainEventsCleared => {
        //             // Render on the swapchain
        //             renderer.render_frame();
        //         }
        //         _ => {}
        //     }
        // });
    }

    pub fn render(&self, window: &Window) {}
}

impl ApplicationHandler for Editor {
    fn new_events(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, cause: winit::event::StartCause) {
        let window_attributes = Window::default_attributes().with_title("Vector Engine").with_inner_size(LogicalSize::new(960.0, 540.0));

        self.window = Some(event_loop.create_window(window_attributes).unwrap());

        self.renderer = Some(Renderer::new_with_window(self.window.as_ref().unwrap()));
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn window_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, window_id: winit::window::WindowId, event: WindowEvent) {
        println!("{event:?}");

        match event {
            WindowEvent::RedrawRequested => {
                if self.window.is_none() {
                    return;
                }
            }
            _ => {}
        }
    }
}

fn main() {
    env::set_var("RUST_LOG", "info");
    env::set_var("RUST_BACKTRACE", "1");

    pretty_env_logger::init();

    ffmpeg_sidecar::download::auto_download().unwrap();

    let editor = Editor::new();
    editor.open();

    // let (watcher_sender, watcher_receiver) = channel::<()>();

    // let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| match res {
    //     Ok(_event) => {
    //         watcher_sender.send(()).unwrap();
    //     }
    //     _ => {}
    // })
    // .unwrap();

    // watcher.watch(Path::new(r#"D:\Vector Engine\playground"#), RecursiveMode::Recursive).unwrap();

    // thread::spawn(move || loop {
    //     info!("Rendering...");

    //     let now = Instant::now();

    //     let mut output = FfmpegCommand::new()
    //         .args(["-f", "rawvideo", "-pix_fmt", "rgba", "-s", "1920x1080", "-r", "30"])
    //         .input("-")
    //         .args(["-c:v", "libx265", "-pix_fmt", "yuva420p"])
    //         .args(["-y", "renders/render.mp4"])
    //         .spawn()
    //         .unwrap();

    //     let mut stdin = output.take_stdin().unwrap();

    //     let mut renderer = Renderer::new();

    //     let mut clip_loader = ClipLoader::new();
    //     let clip = clip_loader.get(&String::from("project.ts"), &renderer).unwrap();
    //     let mut clip = &mut *clip.borrow_mut();

    //     let mut total_frame_time = 0;
    //     let mut total_frames = 0;

    //     for i in 0..60 {
    //         match &mut clip {
    //             Clips::ScriptClip(ref mut clip) => {
    //                 clip.set_frame(i);

    //                 let frame_now = Instant::now();
    //                 let bytes = clip.render_to_raw(&mut renderer, &mut clip_loader, 1920, 1080);
    //                 total_frame_time += frame_now.elapsed().as_millis();
    //                 total_frames += 1;

    //                 stdin.write_all(&bytes).ok();
    //             }
    //             _ => {}
    //         }
    //     }

    //     info!("Average frame time {}ms", total_frame_time / total_frames);

    //     info!("Render fully at {}ms", now.elapsed().as_millis());

    //     watcher_receiver.recv().unwrap();
    // })
    // .join()
    // .unwrap();
}
