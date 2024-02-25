use deno_ast::MediaType;
use deno_ast::ParseParams;
use deno_ast::SourceTextInfo;
use deno_core::error::type_error;
use deno_core::error::AnyError;
use deno_core::futures::FutureExt;
use deno_core::op;
use deno_core::op2;
use deno_core::v8;
use deno_core::{FastString, OpState};
use log::info;
use std::cell::RefCell;
use std::env;
use std::mem;
use std::path::Path;
use std::ptr::null;
use std::rc::Rc;

pub struct RuntimeState {
    vertices: Vec<f32>,
}

pub struct Runtime {
    runtime: tokio::runtime::Runtime,
}

impl Runtime {
    pub fn create() -> Runtime {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

        Runtime { runtime }
    }

    async fn test_hidden(self: &Runtime) -> Rc<RefCell<RuntimeState>> {
        deno_core::extension!(runtime_extension, ops = [op_set_vertices, op_callback_test], options = { state: Rc<RefCell<RuntimeState>> }, state = |state, options| {
            state.put::<Rc<RefCell<RuntimeState>>>(options.state);
        });

        let mut state = Rc::new(RefCell::new(RuntimeState { vertices: vec![] }));

        let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
            module_loader: Some(Rc::new(TsModuleLoader)),
            extensions: vec![runtime_extension::init_ops_and_esm(state.clone())],
            ..Default::default()
        });

        js_runtime
            .execute_script("vector-engine/runtime.js", deno_core::FastString::from_static(include_str!("./runtime.js")))
            .unwrap();

        info!("Path: {}", deno_core::resolve_path("../example.ts", Path::new(env::current_dir().unwrap().as_path())).unwrap());

        let main_module = deno_core::resolve_path("../example.ts", Path::new(env::current_dir().unwrap().as_path())).unwrap();
        let mod_id = js_runtime.load_main_module(&main_module, None).await.unwrap();
        let result = js_runtime.mod_evaluate(mod_id);

        js_runtime.run_event_loop(false).await.unwrap();

        result.await.unwrap().unwrap();

        return state;
    }

    pub fn test(self: &mut Runtime) -> Vec<f32> {
        let mut future = self.test_hidden();
        let result = self.runtime.block_on(future);
        let result_borrow = result.borrow();

        return result_borrow.vertices.clone();
    }

    pub fn execute_clip(&mut self, script: &String) -> Vec<f32> {
        vec![-0.5, 0.5, 0.0, -0.5, 0.5, 0.5]
    }
}

#[op2]
fn op_callback_test<'a>(scope: &mut v8::HandleScope<'a>, callback_value: v8::Local<'a, v8::Value>) -> Result<(), AnyError> {
    info!("{}", callback_value.to_rust_string_lossy(scope));

    let callback = v8::Local::<v8::Function>::try_from(callback_value).map_err(|_| type_error("Invalid argument"))?;

    let this = v8::undefined(scope).into();
    let result = callback.call(scope, this, &[]).unwrap();
    let generator = v8::Local::<v8::Object>::try_from(result).unwrap();
    let next_key = v8::String::new(scope, "next").unwrap().into();
    let next_value = generator.get(scope, next_key).unwrap();
    let next = v8::Local::<v8::Function>::try_from(next_value).unwrap();

    info!("{}", next.to_rust_string_lossy(scope));

    next.call(scope, generator.into(), &[]);

    Ok(())
}

#[op]
fn op_set_vertices(state: &mut OpState, vertices: Vec<f32>) -> Result<(), AnyError> {
    let mut runtime_state = state.borrow_mut::<Rc<RefCell<RuntimeState>>>().borrow_mut();
    runtime_state.vertices = vertices;

    Ok(())
}

struct TsModuleLoader;

impl deno_core::ModuleLoader for TsModuleLoader {
    fn resolve(&self, specifier: &str, referrer: &str, _kind: deno_core::ResolutionKind) -> Result<deno_core::ModuleSpecifier, deno_core::error::AnyError> {
        deno_core::resolve_import(specifier, referrer).map_err(|e| e.into())
    }

    fn load(&self, module_specifier: &deno_core::ModuleSpecifier, _maybe_referrer: Option<&deno_core::ModuleSpecifier>, _is_dyn_import: bool) -> std::pin::Pin<Box<deno_core::ModuleSourceFuture>> {
        let module_specifier = module_specifier.clone();
        async move {
            let path = module_specifier.to_file_path().unwrap();

            let media_type = MediaType::from_path(&path);
            let (module_type, should_transpile) = match media_type {
                MediaType::JavaScript | MediaType::Mjs | MediaType::Cjs => (deno_core::ModuleType::JavaScript, false),
                MediaType::Jsx => (deno_core::ModuleType::JavaScript, true),
                MediaType::TypeScript | MediaType::Mts | MediaType::Cts | MediaType::Dts | MediaType::Dmts | MediaType::Dcts | MediaType::Tsx => (deno_core::ModuleType::JavaScript, true),
                MediaType::Json => (deno_core::ModuleType::Json, false),
                _ => panic!("Unknown extension {:?}", path.extension()),
            };

            let code = std::fs::read_to_string(&path)?;
            let code = if should_transpile {
                let parsed = deno_ast::parse_module(ParseParams {
                    specifier: module_specifier.to_string(),
                    text_info: SourceTextInfo::from_string(code),
                    media_type,
                    capture_tokens: false,
                    scope_analysis: false,
                    maybe_syntax: None,
                })?;
                parsed.transpile(&Default::default())?.text
            } else {
                code
            };

            let module = deno_core::ModuleSource::new(module_type, FastString::from(code), &module_specifier);

            Ok(module)
        }
        .boxed_local()
    }
}
