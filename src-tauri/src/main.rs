// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clips;
mod renderer;
mod runtime;

use log::info;
use std::fs::read_to_string;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::{env, fs::File, io::BufWriter, sync::Mutex};
use tauri::{Manager, State};

use clips::{Clip, ScriptClip};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use renderer::Renderer;
use runtime::ScriptClipRuntime;

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
    env::set_var("RUST_BACKTRACE", "1");
    pretty_env_logger::init();

    let (sender, receiver) = channel::<Command>();

    let thread_sender = sender.clone();
    thread::spawn(move || {
        let timeline = Timeline {};
        let mut clips: Vec<Box<dyn Clip>> = vec![];
        let renderer = Renderer::create();

        let test_clip = ScriptClip::new(include_str!("../../playground/project.ts").to_string());
        clips.push(Box::new(test_clip));

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| match res {
            Ok(event) => match event.kind {
                EventKind::Modify(_) => {
                    thread_sender.send(Command::PlaygroundUpdate).unwrap();
                }
                _ => {}
            },
            _ => {}
        })
        .unwrap();

        watcher.watch(Path::new(r#"D:\Vector Engine\playground"#), RecursiveMode::Recursive).unwrap();

        loop {
            let command = receiver.recv().unwrap();

            match command {
                Command::Preview(frame, response_sender) => {
                    let clip = &mut clips[0];

                    clip.set_frame(frame);

                    response_sender.send(clip.render(&renderer)).unwrap();
                }
                Command::PlaygroundUpdate => {
                    info!("Updating clip!");

                    let clip = ScriptClip::new(read_to_string(Path::new(r#"D:\Vector Engine\playground"#)).unwrap());

                    clips[0] = Box::new(clip);
                }
                Command::Render => {
                    let mut clip = ScriptClip::new(include_str!("../../playground/project.ts").to_string());

                    for frame in 0..60 {
                        clip.set_frame(frame);

                        let render = clip.render(&renderer);

                        let mut file = File::create(format!("../renders/render_{:0>2}.png", frame)).unwrap();
                        file.write_all(&render).unwrap();
                    }
                }
            }
        }
    });

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
