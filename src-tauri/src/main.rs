// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clips;
mod renderer;
mod runtime;

use log::{info, warn};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::io::{Cursor, Write};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use std::{env, fs::File, io::BufWriter, sync::Mutex};
use tauri::{Manager, State, Url};

use clips::{ClipLoader, Clips, ScriptClip};
use renderer::Renderer;

struct Timeline {}

#[tauri::command]
fn render(sender: State<Sender<Command>>) {
    sender.send(Command::Render).unwrap();
}

pub enum Command {
    Preview(u32, Sender<Vec<u8>>),
    Render,
    PlaygroundUpdate,
}

fn main() {
    env::set_var("RUST_LOG", "info");
    // env::set_var("RUST_BACKTRACE", "1");

    pretty_env_logger::init();

    let (sender, receiver) = channel::<Command>();

    let watch_thread_sender = sender.clone();
    let preview_thread_sender = sender.clone();
    tauri::Builder::default()
        .manage(sender)
        .invoke_handler(tauri::generate_handler![render])
        .register_uri_scheme_protocol("preview", move |_app, req| {
            let url: Url = req.uri().parse().unwrap();

            let (response_sender, response_receiver) = channel();

            let queries: HashMap<String, String> = url.query_pairs().into_owned().collect();

            preview_thread_sender
                .send(Command::Preview(u32::from_str_radix(queries.get("frame").unwrap(), 10).unwrap(), response_sender))
                .unwrap();

            let bytes = response_receiver.recv().unwrap();

            let mut encoded_bytes: Vec<u8> = vec![];

            let mut encoder = png::Encoder::new(&mut encoded_bytes, 480, 270);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);

            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&bytes).unwrap();
            writer.finish().unwrap();

            tauri::http::ResponseBuilder::new()
                .header("Access-Control-Allow-Origin", "*")
                .header("Origin", "*")
                .mimetype("image/png")
                .header("Content-Length", encoded_bytes.len())
                .status(200)
                .body(encoded_bytes)
        })
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.handle().get_window("main").unwrap();

                window.open_devtools();
                window.close_devtools();
            }

            thread::spawn(move || {
                let mut renderer = Renderer::create(1920, 1080);

                let mut preview_renderer = Renderer::create(480, 270);

                let clip_loader = ClipLoader::new();

                let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| match res {
                    Ok(_event) => {
                        watch_thread_sender.send(Command::PlaygroundUpdate).unwrap();
                    }
                    _ => {}
                })
                .unwrap();

                watcher.watch(Path::new(r#"D:\Vector Engine\playground"#), RecursiveMode::Recursive).unwrap();

                loop {
                    let command = receiver.recv().unwrap();

                    match command {
                        Command::Preview(frame, response_sender) => {
                            let clip = clip_loader.get_new(String::from("project.ts")).unwrap();

                            match clip {
                                Clips::ScriptClip(mut clip) => {
                                    clip.set_frame(frame);

                                    let render = clip.render(&mut preview_renderer, &clip_loader);

                                    response_sender.send(render).unwrap();
                                }
                            }
                        }
                        Command::PlaygroundUpdate => {
                            // let old_clip = clips.remove(&String::from("project.ts"));
                            // drop(old_clip);

                            // clips.insert(
                            //     String::from("project.ts"),
                            //     Arc::new(RefCell::new(Clips::ScriptClip(ScriptClip::new(
                            //         read_to_string(Path::new(r#"D:\Vector Engine\playground\project.ts"#)).unwrap(),
                            //     )))),
                            // );
                        }
                        Command::Render => {
                            for frame in 0..240 {
                                let clip = clip_loader.get_new(String::from("project.ts")).unwrap();

                                match clip {
                                    Clips::ScriptClip(mut clip) => {
                                        clip.set_frame(frame);

                                        let bytes = clip.render(&mut renderer, &clip_loader);

                                        thread::spawn(move || {
                                            let file = File::create(format!("D:/Vector Engine/renders/render_{:0>3}.png", frame)).unwrap();
                                            let mut file_writer = BufWriter::new(file);

                                            let mut encoder = png::Encoder::new(&mut file_writer, 1920, 1080);
                                            encoder.set_color(png::ColorType::Rgba);
                                            encoder.set_depth(png::BitDepth::Eight);

                                            let mut writer = encoder.write_header().unwrap();
                                            writer.write_image_data(&bytes).unwrap();
                                            writer.finish().unwrap();
                                        });
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// app.run(|app_handle, event| match event {
//     tauri::RunEvent::ExitRequested { .. } => {
//         let project_mutex = app_handle.state::<Mutex<Project>>();
//         let mut project = project_mutex.lock().unwrap();

//         destroy_renderer(&mut project.renderer).unwrap();
//     }
//     _ => {}
// });
