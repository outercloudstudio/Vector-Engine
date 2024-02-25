// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod renderer;
mod runtime;

use log::info;
use std::{env, fs::File, io::BufWriter, sync::Mutex};
use tauri::{Manager, State};

use renderer::Renderer;
use runtime::Runtime;

struct Timeline {}

struct Clip {}

struct Project {
    timeline: Timeline,
    clips: Vec<Clip>,
    renderer: Renderer,
    runtime: Runtime,
}

#[tauri::command]
fn preview(project_mutex: State<Mutex<Project>>) -> Vec<u8> {
    println!("Rendering preview...");

    let mut project = project_mutex.lock().unwrap();
    let bytes = project.renderer.render();

    let mut encoded_bytes: Vec<u8> = vec![];
    let mut encoder = png::Encoder::new(&mut encoded_bytes, 512, 512);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&bytes).unwrap();
    writer.finish().unwrap();

    return encoded_bytes;
}

fn main() {
    env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();

    let timeline = Timeline {};
    let clips = vec![];
    let renderer = Renderer::create();
    let runtime = Runtime::create();

    runtime.test();

    let project = Project { timeline, clips, renderer, runtime };
    let project_mutex = Mutex::new(project);

    tauri::Builder::default()
        .manage(project_mutex)
        .invoke_handler(tauri::generate_handler![preview])
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
