// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{path::Path, thread, time::Duration};

use notify::{Event, RecursiveMode, Watcher};
use std::fs;
use tauri::{AppHandle, Manager};

extern crate swc_common;
extern crate swc_ecma_parser;
use swc::config::{Config, IsModule, JscConfig, Options};
use swc_common::sync::Lrc;
use swc_common::{
	errors::{ColorConfig, Handler},
	FileName, SourceMap,
};
use swc_common::{Globals, Mark, GLOBALS};
use swc_ecma_ast::{EsVersion, Program};
use swc_ecma_parser::Syntax;

fn project_update(event: Event, app_handle: &AppHandle, project_path: &str) {
	println!("event: {:?}", event);

	let source_map: Lrc<SourceMap> = Default::default();
	let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(source_map.clone()));
	let compiler = swc::Compiler::new(source_map.clone());
	let globals = Globals::default();

	GLOBALS.set(&globals, || {
		let paths = fs::read_dir(project_path).unwrap();

		for path_result in paths {
			let path_buffer = path_result.unwrap().path();
			let path = path_buffer.as_path();

			println!("Name: {}", path.display());

			source_map.load_file(path).unwrap();
		}

		println!("Files {}", source_map.files().as_slice().len());

		let mut programs: Vec<Program> = Vec::new();

		for file in source_map.files().as_slice() {
			println!("File Lines {}", file.count_lines());

			match compiler.parse_js(
				file.clone(),
				&handler,
				EsVersion::Es5,
				Syntax::Typescript(Default::default()),
				IsModule::Unknown,
				None,
			) {
				Err(error) => println!("{}", error),
				Ok(program) => {
					let program = compiler.transform(
						&handler,
						program,
						false,
						swc_ecma_transforms_typescript::strip(Mark::new()),
					);

					programs.push(program);

					println!("Finished Transform!");

					// match compiler.process_js(
					// 	&handler,
					// 	program,
					// 	&Options {
					// 		config: Config {
					// 			jsc: JscConfig {
					// 				target: Some(EsVersion::Es5),
					// 				syntax: Some(Syntax::Es(Default::default())),
					// 				..Default::default()
					// 			},
					// 			..Default::default()
					// 		},
					// 		..Default::default()
					// 	},
					// ) {
					// 	Err(error) => println!("{}", error),
					// 	Ok(output) => print!("{}", output.code),
					// }
				}
			};

			println!("Finished File!");
		}

		for program in programs {
			println!("Compiling Program ");

			match compiler.process_js(
				&handler,
				program,
				&Options {
					config: Config {
						jsc: JscConfig {
							target: Some(EsVersion::Es5),
							syntax: Some(Syntax::Es(Default::default())),
							..Default::default()
						},
						..Default::default()
					},
					..Default::default()
				},
			) {
				Err(error) => println!("{}", error),
				Ok(output) => print!("{}", output.code),
			}

			println!("Finished File!");
		}
	});

	app_handle
		.emit_all("test", String::from("Worked!"))
		.unwrap();
}

fn main() {
	let project_path = "D:/Vector Engine Projects/Tauri/";

	tauri::Builder::default()
		.setup(move |app| {
			let app_handle = app.handle();

			thread::spawn(move || {
				let mut watcher = notify::recommended_watcher(move |res| match res {
					Ok(event) => project_update(event, &app_handle, project_path),
					Err(e) => println!("watch error: {:?}", e),
				})
				.unwrap();

				watcher
					.watch(Path::new(project_path), RecursiveMode::Recursive)
					.unwrap();

				loop {
					thread::sleep(Duration::from_millis(1));
				}
			});

			println!("Finished Setup!");

			// let test_code = "alert('Worked!')";

			// app.emit_all("test", test_code).unwrap();

			Ok(())
		})
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
