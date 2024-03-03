// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clips;
mod renderer;
mod runtime;

use log::{info, warn};
use std::fs::read_to_string;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;
use std::{env, fs::File, io::BufWriter, sync::Mutex};
use tauri::{Manager, State};

use clips::{Clip, Clips, ScriptClip};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use renderer::Renderer;

struct Timeline {}

#[tauri::command]
fn preview(frame: u32, sender: State<Sender<Command>>) -> Vec<u8> {
    let (response_sender, response_receiver) = channel();

    sender.send(Command::Preview(frame, response_sender)).unwrap();

    response_receiver.recv().unwrap()
}

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

    let thread_sender = sender.clone();
    tauri::Builder::default()
        .manage(sender)
        .invoke_handler(tauri::generate_handler![preview, render])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.handle().get_window("main").unwrap();

                window.open_devtools();
                window.close_devtools();
            }

            thread::spawn(move || {
                let mut renderer = Renderer::create();

                let mut clips: Vec<Clips> = Vec::new();

                clips.push(Clips::ScriptClip(ScriptClip::new(include_str!("../../playground/project.ts").to_string())));

                let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| match res {
                    Ok(_event) => {
                        thread_sender.send(Command::PlaygroundUpdate).unwrap();
                    }
                    _ => {}
                })
                .unwrap();

                watcher.watch(Path::new(r#"D:\Vector Engine\playground"#), RecursiveMode::Recursive).unwrap();

                loop {
                    let command = receiver.recv().unwrap();

                    match command {
                        Command::Preview(frame, response_sender) => {
                            let clip = &mut clips[0];

                            match clip {
                                Clips::ScriptClip(clip) => {
                                    clip.set_frame(frame);

                                    let render = clip.render(&mut renderer);

                                    response_sender.send(render).unwrap();
                                }
                            }
                        }
                        Command::PlaygroundUpdate => {
                            let old_clip = clips.remove(0);
                            drop(old_clip);

                            clips.push(Clips::ScriptClip(ScriptClip::new(read_to_string(Path::new(r#"D:\Vector Engine\playground\project.ts"#)).unwrap())));
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
