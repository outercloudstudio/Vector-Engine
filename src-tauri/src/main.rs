// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clips;
mod renderer;
mod runtime;

use log::info;
use std::io::Write;
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
fn preview(sender: State<Sender<Command>>) -> Vec<Vec<u8>> {
    let (response_sender, response_receiver) = channel();

    sender.send(Command::Preview(response_sender)).unwrap();

    let bytes = response_receiver.recv().unwrap();

    let mut file = File::create("../renders/render.png").unwrap();
    file.write_all(&bytes).unwrap();

    vec![bytes]
}

#[tauri::command]
fn render(sender: State<Sender<Command>>) {
    sender.send(Command::Render).unwrap();
}

pub enum Command {
    Preview(Sender<Vec<u8>>),
    Render,
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
            Command::Preview(response_sender) => {
                let mut clip = ScriptClip::new(String::from(
                    r#"
clip(function* (){
    console.log(':D')

    const rect = add(new Rect(
        new Vector2(0, 0),
        new Vector2(200, 200),
    ))

    while(true) {
        yield null;

        rec.position.x += 1
    }
})
                    "#,
                ));

                clip.set_frame(0);

                response_sender.send(clip.render(&project)).unwrap();
            }
            Command::Render => {
                let mut clip = ScriptClip::new(String::from(
                    r#"
clip(function* (){
    console.log(':D')

    const rect = add(new Rect(
        new Vector2(0, 0),
        new Vector2(200, 200),
    ))

    while(true) {
        yield null;

        rec.position.x += 1
    }
})
                    "#,
                ));

                for frame in 0..60 {
                    clip.set_frame(frame);

                    let render = clip.render(&project);

                    let mut file = File::create(format!("../renders/render_{:0>2}.png", frame)).unwrap();
                    file.write_all(&render).unwrap();
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
