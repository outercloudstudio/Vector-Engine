// // Prevents additional console window on Windows in release, DO NOT REMOVE!!
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
// #[tauri::command]
// fn greet(name: &str) -> String {
//     format!("Hello, {}! You've been greeted from Rust!", name)
// }

// fn main() {
//     tauri::Builder::default()
//         .invoke_handler(tauri::generate_handler![greet])
//         .run(tauri::generate_context!())
//         .expect("error while running tauri application");
// }

mod renderer;
mod runtime;

use renderer::RenderContext;
use runtime::Runtime;

struct Timeline {}

struct Clip {}

struct Project {
    timeline: Timeline,
    clips: Vector<Clip>,
    renderer: RenderContext,
    runtime: Runtime,
}

fn main() -> Result<()> {
    let timeline = Timeline {};
    let clips: Vector<Clip> = vec![];
    let renderer = RenderContext {};
    let runtime = Runtime {};

    let mut project = Project { timeline, clips, renderer, runtime };

    Ok(())
}
