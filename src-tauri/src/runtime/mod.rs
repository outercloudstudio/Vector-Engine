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
use std::rc::Rc;

pub struct ClipRuntimeState {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub last_index: u32,
}

impl Clone for ClipRuntimeState {
    fn clone(&self) -> Self {
        Self {
            vertices: self.vertices.clone(),
            indices: self.indices.clone(),
            last_index: self.last_index.clone(),
        }
    }
}

pub struct Runtime {
    runtime: tokio::runtime::Runtime,
}

impl Runtime {
    pub fn create() -> Runtime {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

        Runtime { runtime }
    }

    async fn execute_clip_script(self: &Runtime, script: &String) -> ClipRuntimeState {
        deno_core::extension!(runtime_extension, ops = [op_set_vertices, op_callback_test, op_add_element], options = { state: Rc<RefCell<ClipRuntimeState>> }, state = |state, options| {
            state.put::<Rc<RefCell<ClipRuntimeState>>>(options.state);
        });

        let mut state = Rc::new(RefCell::new(ClipRuntimeState {
            vertices: Vec::new(),
            indices: Vec::new(),
            last_index: 0,
        }));

        let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
            module_loader: Some(Rc::new(TsModuleLoader)),
            extensions: vec![runtime_extension::init_ops_and_esm(state.clone())],
            ..Default::default()
        });

        js_runtime
            .execute_script("vector-engine/runtime.ts", deno_core::FastString::from(transpile_ts(String::from(include_str!("./runtime.ts")))))
            .unwrap();

        js_runtime.execute_script("project/clip.ts", deno_core::FastString::from(transpile_ts(script.clone()))).unwrap();

        // info!("Path: {}", deno_core::resolve_path("../example.ts", Path::new(env::current_dir().unwrap().as_path())).unwrap());

        // let main_module = deno_core::resolve_path("../example.ts", Path::new(env::current_dir().unwrap().as_path())).unwrap();
        // let mod_id = js_runtime.load_main_module(&main_module, None).await.unwrap();
        // let result = js_runtime.mod_evaluate(mod_id);

        js_runtime.run_event_loop(false).await.unwrap();

        // result.await.unwrap().unwrap();

        return state.borrow().clone();
    }

    pub fn execute_clip(&mut self, script: &String) -> ClipRuntimeState {
        let mut future = self.execute_clip_script(script);
        let result = self.runtime.block_on(future);

        return result;
    }
}

fn transpile_ts(code: String) -> String {
    let parsed = deno_ast::parse_module(ParseParams {
        specifier: String::from(""),
        text_info: SourceTextInfo::from_string(code),
        media_type: MediaType::TypeScript,
        capture_tokens: false,
        scope_analysis: false,
        maybe_syntax: None,
    })
    .unwrap();

    parsed.transpile(&Default::default()).unwrap().text
}

pub fn deserialize_number(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> f32 {
    v8::Local::<v8::Number>::try_from(value).unwrap().value() as f32
}

pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn deserialize(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> Vector2 {
        let object = v8::Local::<v8::Object>::try_from(value).unwrap();

        let x_key = v8::String::new(scope, "x").unwrap().into();
        let x_value = object.get(scope, x_key).unwrap();

        let y_key = v8::String::new(scope, "y").unwrap().into();
        let y_value = object.get(scope, y_key).unwrap();

        Vector2 {
            x: deserialize_number(scope, x_value),
            y: deserialize_number(scope, y_value),
        }
    }
}

pub struct Rect {
    pub position: Vector2,
    pub size: Vector2,
}

impl Rect {
    pub fn deserialize(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> Rect {
        let object = v8::Local::<v8::Object>::try_from(value).unwrap();

        let position_key = v8::String::new(scope, "position").unwrap().into();
        let position_value = object.get(scope, position_key).unwrap();

        let size_key = v8::String::new(scope, "size").unwrap().into();
        let size_value = object.get(scope, size_key).unwrap();

        Rect {
            position: Vector2::deserialize(scope, position_value),
            size: Vector2::deserialize(scope, size_value),
        }
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
    let mut runtime_state = state.borrow_mut::<Rc<RefCell<ClipRuntimeState>>>().borrow_mut();
    runtime_state.vertices = vertices;

    Ok(())
}

#[op2]
fn op_add_element<'a>(state: &mut OpState, scope: &mut v8::HandleScope<'a>, value: v8::Local<'a, v8::Value>) -> Result<(), AnyError> {
    let mut clip_state = state.borrow_mut::<Rc<RefCell<ClipRuntimeState>>>().borrow_mut();

    let object = v8::Local::<v8::Object>::try_from(value).unwrap();

    let type_key = v8::String::new(scope, "type").unwrap().into();
    let type_value = object.get(scope, type_key).unwrap();
    let type_string = v8::Local::<v8::String>::try_from(type_value).unwrap().to_rust_string_lossy(scope);

    if type_string == "Rect" {
        let rect = Rect::deserialize(scope, value);

        clip_state.vertices.push((rect.position.x) / 512.0);
        clip_state.vertices.push(-(rect.position.y) / 512.0);

        clip_state.vertices.push((rect.position.x) / 512.0);
        clip_state.vertices.push(-(rect.position.y + rect.size.y) / 512.0);

        clip_state.vertices.push((rect.position.x + rect.size.y) / 512.0);
        clip_state.vertices.push(-(rect.position.y + rect.size.y) / 512.0);

        clip_state.vertices.push((rect.position.x + rect.size.y) / 512.0);
        clip_state.vertices.push(-(rect.position.y) / 512.0);

        let mut indices = vec![
            clip_state.last_index + 0,
            clip_state.last_index + 1,
            clip_state.last_index + 2,
            clip_state.last_index + 2,
            clip_state.last_index + 3,
            clip_state.last_index + 0,
        ];

        clip_state.indices.append(&mut indices);

        clip_state.last_index += 4;
    }

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
