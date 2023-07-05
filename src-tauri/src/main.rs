// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{path::Path, thread, time::Duration};

use notify::{RecursiveMode, Watcher};
use tauri::Manager;

fn main() {
	tauri::Builder::default()
		.setup(|app| {
			println!("Setup app!");

			let app_handle = app.handle();

			thread::spawn(move || {
				let mut watcher = notify::recommended_watcher(move |res| match res {
					Ok(event) => {
						println!("event: {:?}", event);

						app_handle
							.emit_all("test", String::from("Worked!"))
							.unwrap();

						return;
					}
					Err(e) => println!("watch error: {:?}", e),
				})
				.unwrap();

				watcher
					.watch(
						Path::new("D:/Vector Engine Projects/Tauri/"),
						RecursiveMode::Recursive,
					)
					.unwrap();

				loop {
					thread::sleep(Duration::from_millis(1));
				}
			});

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
