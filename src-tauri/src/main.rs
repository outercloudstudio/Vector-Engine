use deno_core::error::AnyError;
use deno_core::{op, Extension};
use std::env;
use std::path::Path;
use std::rc::Rc;

deno_core::extension!(runtime_extension, ops = [op_read_file, op_write_file, op_remove_file]);

async fn run_js(file_path: &str) -> Result<(), AnyError> {
    let main_module = deno_core::resolve_path(file_path, Path::new(env::current_dir().unwrap().as_path()))?;

    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        extensions: vec![runtime_extension::init_ops()],
        ..Default::default()
    });

    js_runtime
        .execute_script("[runjs:runtime.js]", deno_core::FastString::from_static(include_str!("./runtime.js")))
        .unwrap();

    let mod_id = js_runtime.load_main_module(&main_module, None).await?;
    let result = js_runtime.mod_evaluate(mod_id);
    js_runtime.run_event_loop(false).await?;
    result.await?
}

fn main() {
    println!("Path: {}", env::current_dir().unwrap().to_str().unwrap());

    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

    if let Err(error) = runtime.block_on(run_js("./example.js")) {
        eprintln!("error: {}", error);
    }
}

#[op]
async fn op_read_file(path: String) -> Result<String, AnyError> {
    let contents = tokio::fs::read_to_string(path).await?;
    Ok(contents)
}

#[op]
async fn op_write_file(path: String, contents: String) -> Result<(), AnyError> {
    tokio::fs::write(path, contents).await?;
    Ok(())
}

#[op]
fn op_remove_file(path: String) -> Result<(), AnyError> {
    std::fs::remove_file(path)?;
    Ok(())
}
