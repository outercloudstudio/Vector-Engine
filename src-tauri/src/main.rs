// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod renderer;
mod runtime;

use anyhow::{anyhow, Result};
use std::sync::Mutex;
use vulkanalia::vk::InstanceV1_0;

use renderer::{create_renderer, Renderer};
use runtime::Runtime;
use tauri::{generate_context, Manager, RunEvent, State};

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

    // render(&mut project.renderer).unwrap();

    return vec![];
}

fn main() -> Result<()> {
    let timeline = Timeline {};
    let clips: Vec<Clip> = vec![];
    let renderer = create_renderer()?;

    unsafe {
        renderer.instance.get_physical_device_memory_properties(renderer.context.physical_device);

        println!("[DEBUG] Got memory! 2");
    }

    let runtime = Runtime {};

    let project = Project { timeline, clips, renderer, runtime };
    let project_mutex = Mutex::new(project);

    // let app = tauri::Builder::default()
    //     .manage(project_mutex)
    //     .invoke_handler(tauri::generate_handler![preview])
    //     .setup(|app| {
    //         #[cfg(debug_assertions)] // only include this code on debug builds
    //         {
    //             let window = app.get_window("main").unwrap();
    //             window.open_devtools();
    //             window.close_devtools();
    //         }
    //         Ok(())
    //     })
    //     .build(generate_context!())
    //     .expect("error while running tauri application");

    // app.run(|app_handle, event| match event {
    //     tauri::RunEvent::ExitRequested { .. } => {
    //         let project_mutex = app_handle.state::<Mutex<Project>>();
    //         let mut project = project_mutex.lock().unwrap();

    //         destroy_renderer(&mut project.renderer).unwrap();
    //     }
    //     _ => {}
    // });

    Ok(())
}
