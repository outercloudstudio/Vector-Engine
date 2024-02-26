// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clips;
mod renderer;
mod runtime;

use log::info;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::{env, fs::File, io::BufWriter, sync::Mutex};
use tauri::{Manager, State};

use clips::{Clip, ScriptClip};
use renderer::Renderer;
use runtime::ScriptClipRuntime;

struct Timeline {}

struct Project {
    timeline: Timeline,
    clips: Vec<Box<dyn Clip>>,
    renderer: Renderer,
}

#[tauri::command]
fn preview(sender: State<Sender<Command>>) -> Vec<u8> {
    sender.send(Command::Preview).unwrap();

    Vec::new()
}

pub enum Command {
    Preview,
}

fn main() {
    env::set_var("RUST_LOG", "info");
    env::set_var("RUST_BACKTRACE", "1");
    pretty_env_logger::init();

    let (sender, receiver) = channel::<Command>();

    thread::spawn(move || {
        let timeline = Timeline {};
        let clips = vec![];
        let renderer = Renderer::create();

        let mut project = Project { timeline, clips, renderer };

        let command = receiver.recv().unwrap();

        match command {
            Command::Preview => info!("Preview!"),
        }

        match command {
            Command::Preview => {
                let mut clip = ScriptClip::new(String::from(
                    r#"
                    clip(function* (){
                        console.log(':D')
                    
                        add(new Rect(
                            new Vector2(0, 0),
                            new Vector2(200, 200),
                        ))
                        
                        add(new Rect(
                            new Vector2(0, 200),
                            new Vector2(100, 100),
                        ))
                    
                        yield null;
                    })
                    "#,
                ));

                clip.set_frame(0);

                clip.render(&project);

                info!("Previewed!");
            }
        }
    });

    tauri::Builder::default()
        .manage(sender)
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
