// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::Path;

use notify::{RecommendedWatcher, RecursiveMode, Result, Watcher};
use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
	format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
	tauri::Builder::default()
		.invoke_handler(tauri::generate_handler![greet])
		.setup(|app| {
			println!("Setup app!");

			let mut watcher = notify::recommended_watcher(|res| match res {
				Ok(event) => println!("event: {:?}", event),
				Err(e) => println!("watch error: {:?}", e),
			})?;

			watcher.watch(Path::new("."), RecursiveMode::Recursive)?;

			// let test_code = "alert('Worked!')";

			// app.emit_all("test", test_code).unwrap();

			Ok(())
		})
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
