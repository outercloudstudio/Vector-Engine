// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod renderer;
mod runtime;

use anyhow::{anyhow, Result};
use std::sync::Mutex;
use vulkanalia::prelude::v1_0::*;

use renderer::{create_renderer, get_test_stuff, Renderer};
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
    // let renderer = create_renderer()?;
    let (instance, physical_device) = get_test_stuff()?;

    unsafe {
        // println!("[DEBUG] Image test?");

        // renderer.device.get_image_memory_requirements(renderer.context.target_image);

        println!("[DEBUG] Getting properties!");

        instance.get_physical_device_properties(physical_device);

        println!("[DEBUG] Worked!");

        // println!("[DEBUG] Getting memory! {}", physical_device.as_raw());

        // instance.get_physical_device_memory_properties(physical_device);

        // println!("[DEBUG] Got memory! 2");
    }

    let runtime = Runtime {};

    // let project = Project { timeline, clips, renderer, runtime };
    // let project_mutex = Mutex::new(project);

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
