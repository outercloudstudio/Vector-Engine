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

    tauri::Builder::default()
        .manage(sender)
        .invoke_handler(tauri::generate_handler![preview, render])
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.handle().get_window("main").unwrap();

                window.open_devtools();
                window.close_devtools();
            }

            let handle = app.app_handle();

            thread::spawn(move || {
                let renderer = Renderer::create();

                let mut clips = Vec::new();

                let mut test_clip = ScriptClip::new(include_str!("../../playground/project.ts").to_string());

                clips.push(test_clip);

                let mut frame = 0;

                loop {
                    clips[0].set_frame(frame);

                    handle.emit_all("render", clips[0].render(&renderer)).unwrap();

                    frame += 1;

                    if frame == 60 {
                        let old_clip = clips.remove(0);

                        drop(old_clip);

                        clips.push(ScriptClip::new(include_str!("../../playground/project.ts").to_string()));

                        frame = 0;
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
