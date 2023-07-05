// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
	path::Path,
	sync::{Arc, Mutex},
};

use notify::{RecursiveMode, Watcher};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
	format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
	let watcher = notify::recommended_watcher(|res| match res {
		Ok(event) => println!("event: {:?}", event),
		Err(e) => println!("watch error: {:?}", e),
	})
	.unwrap();

	let watcher_arc_mutex = Arc::new(Mutex::new(watcher));

	let watcher_arc_mutex_setup_reference = watcher_arc_mutex.clone();

	tauri::Builder::default()
		.invoke_handler(tauri::generate_handler![greet])
		.setup(move |_app| {
			println!("Setup app!");

			let mut watcher_lock = watcher_arc_mutex_setup_reference.lock().unwrap();

			watcher_lock
				.watch(
					Path::new("D:/Vector Engine Projects/Tauri/"),
					RecursiveMode::Recursive,
				)
				.unwrap();

			println!(
				"{}",
				Path::new("D:/Vector Engine Projects/Tauri/")
					.to_str()
					.unwrap()
			);

			println!("Finished Setup!");

			// let test_code = "alert('Worked!')";

			// app.emit_all("test", test_code).unwrap();

			Ok(())
		})
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
