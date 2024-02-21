// // Prevents additional console window on Windows in release, DO NOT REMOVE!!
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
// #[tauri::command]
// fn greet(name: &str) -> String {
//     format!("Hello, {}! You've been greeted from Rust!", name)
// }

// fn main() {
//     tauri::Builder::default()
//         .invoke_handler(tauri::generate_handler![greet])
//         .run(tauri::generate_context!())
//         .expect("error while running tauri application");
// }

use deno_core::{JsRuntime, RuntimeOptions};

fn main() {
    let mut runtime = JsRuntime::new(RuntimeOptions { ..Default::default() });

    runtime
        .execute_script_static(
            "<usage>",
            r#"
// Print helper function, calling Deno.core.print()
function print(value) {
  Deno.core.print(value.toString()+"\n");
}

const arr = [1, 2, 3];
print(arr);
"#,
        )
        .unwrap();
}
