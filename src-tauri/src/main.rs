// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clips;
mod renderer;
mod runtime;

use image::ImageEncoder;
use notify::{Event, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::{env, fs::File, io::BufWriter};
use tauri::{State, Url};

use clips::{ClipLoader, Clips};
use renderer::renderer::Renderer;

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

            let mut encoder = image::codecs::bmp::BmpEncoder::new(&mut encoded_bytes);
            encoder.encode(&bytes, 480, 270, image::ColorType::Rgba8).unwrap();

            tauri::http::ResponseBuilder::new()
                .header("Access-Control-Allow-Origin", "*")
                .header("Origin", "*")
                .mimetype("image/bmp")
                .header("Content-Length", encoded_bytes.len())
                .status(200)
                .body(encoded_bytes)
        })
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                // let window = app.handle().get_window("main").unwrap();

                // window.open_devtools();
                // window.close_devtools();
            }

            thread::spawn(move || {
                let mut renderer = Renderer::new();

                let mut clip_loader = ClipLoader::new();

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
                            let clip = clip_loader.get(&String::from("project.ts"), &renderer).unwrap();

                            let mut clip = &mut *clip.borrow_mut();

                            match &mut clip {
                                Clips::ScriptClip(ref mut clip) => {
                                    clip.set_frame(frame);

                                    let render = clip.render_to_raw(&mut renderer, &clip_loader, 480, 270);

                                    response_sender.send(render).unwrap();
                                }
                                _ => {}
                            }
                        }
                        Command::PlaygroundUpdate => clip_loader.invalidate(&String::from("project.ts")),
                        Command::Render => {
                            for frame in 0..300 {
                                let clip = clip_loader.get_new(&String::from("project.ts"), &renderer).unwrap();

                                match clip {
                                    Clips::ScriptClip(mut clip) => {
                                        clip.set_frame(frame);

                                        let bytes = clip.render_to_raw(&mut renderer, &clip_loader, 1920, 1080);

                                        thread::spawn(move || {
                                            // let file = File::create(format!("D:/Vector Engine/renders/render_{:0>3}.bmp", frame)).unwrap();
                                            // let mut file_writer = BufWriter::new(file);

                                            // let mut encoder = image::codecs::bmp::BmpEncoder::new(&mut file_writer);
                                            // encoder.encode(&bytes, 1920, 1080, image::ColorType::Rgba8).unwrap();

                                            let file = File::create(format!("D:/Vector Engine/renders/render_{:0>3}.png", frame)).unwrap();
                                            let mut file_writer = BufWriter::new(file);

                                            let encoder = image::codecs::png::PngEncoder::new(&mut file_writer);
                                            encoder.write_image(&bytes, 1920, 1080, image::ColorType::Rgba8).unwrap();
                                        });
                                    }
                                    _ => {}
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
