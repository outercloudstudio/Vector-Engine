// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clips;
mod renderer;
mod runtime;

use log::info;
use std::{env, fs::File, io::BufWriter, sync::Mutex};
use tauri::{Manager, State};

use renderer::Renderer;
use runtime::Runtime;

use crate::clips::{Clip, ScriptClip};

struct Timeline {}

struct Project {
    timeline: Timeline,
    clips: Vec<Box<dyn Clip>>,
    renderer: Renderer,
    runtime: Runtime,
}

#[tauri::command]
fn preview(project_mutex: State<Mutex<Project>>) -> Vec<u8> {
    println!("Rendering preview...");

    let mut project = project_mutex.lock().unwrap();

    let mut clip = ScriptClip::new(String::from(
        r#"
console.log(':D')

const triangle = add(
    new Triangle(
        new Vector2(-0.5, 0.5),
        new Vector2(0.0, -0.5),
        new Vector2(0.5, 0.5),
    )
)

const triangle2 = add(
    new Triangle(
        new Vector2(0.0, -0.5),
        new Vector2(1, -0.4),
        new Vector2(1, 0.5),
    )
)
"#,
    ));

    clip.set_frame(0);

    return clip.render(&mut project);
}

fn main() {
    env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();

    let timeline = Timeline {};
    let clips = vec![];
    let renderer = Renderer::create();
    let runtime = Runtime::create();

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
