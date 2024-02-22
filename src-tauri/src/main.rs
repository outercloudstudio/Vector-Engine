// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod renderer;
mod runtime;

use std::sync::Mutex;

use renderer::{render, RenderContext};
use runtime::Runtime;
use tauri::{Manager, State};

struct Timeline {}

struct Clip {}

struct Project {
    timeline: Timeline,
    clips: Vec<Clip>,
    render_context: RenderContext,
    runtime: Runtime,
}

#[tauri::command]
fn preview(project_mutex: State<Mutex<Project>>) {
    println!("Rendering preview...");

    let project = project_mutex.lock().unwrap();

    render(&project.render_context);
}

fn main() {
    let timeline = Timeline {};
    let clips: Vec<Clip> = vec![];
    let renderer = RenderContext {};
    let runtime = Runtime {};

    let project = Project {
        timeline,
        clips,
        render_context: renderer,
        runtime,
    };
    let project_mutex = Mutex::new(project);

    tauri::Builder::default()
        .manage(project_mutex)
        .invoke_handler(tauri::generate_handler![preview])
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
