use deno_ast::MediaType;
use deno_ast::ParseParams;
use deno_ast::SourceTextInfo;
use deno_core::error::type_error;
use deno_core::error::AnyError;
use deno_core::futures::FutureExt;
use deno_core::op;
use deno_core::op2;
use deno_core::v8;
use deno_core::v8::Global;
use deno_core::Extension;
use deno_core::Op;
use deno_core::{FastString, OpState};
use log::info;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

struct ClipRuntimeState {
    vertices: Vec<f32>,
    indices: Vec<u32>,
    last_index: u32,
    generator_function: Option<Global<v8::Function>>,
    generator: Option<Global<v8::Object>>,
}

pub struct ScriptClipRuntime {
    js_runtime: deno_core::JsRuntime,
    state: Arc<Mutex<ClipRuntimeState>>,
}

impl ScriptClipRuntime {
    pub fn new() -> ScriptClipRuntime {
        let state = Arc::new(Mutex::new(ClipRuntimeState {
            vertices: Vec::new(),
            indices: Vec::new(),
            last_index: 0,
            generator_function: None,
            generator: None,
        }));

        let state_arc = state.clone();

        let runtime_extension = Extension::builder("runtime_extension")
            .ops(vec![op_clip::DECL, op_add_element::DECL])
            .state(|extension_state| {
                extension_state.put::<Arc<Mutex<ClipRuntimeState>>>(state_arc);
            })
            .build();

        let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
            module_loader: Some(Rc::new(TsModuleLoader)),
            extensions: vec![runtime_extension],
            ..Default::default()
        });

        ScriptClipRuntime { js_runtime, state }
    }

    async fn initialize_clip_script(&mut self, script: &String) {
        self.js_runtime
            .execute_script("vector-engine/runtime.ts", deno_core::FastString::from(transpile_ts(String::from(include_str!("./runtime.ts")))))
            .unwrap();

        self.js_runtime.execute_script("project/clip.ts", deno_core::FastString::from(transpile_ts(script.clone()))).unwrap();
        self.js_runtime.run_event_loop(false).await.unwrap();

        let mut state = self.state.lock().unwrap();

        let mut scope = self.js_runtime.handle_scope();

        let generator_function = state.generator_function.clone().unwrap();
        let generator_function: v8::Local<v8::Function> = v8::Local::new(&mut scope, generator_function);

        let this = v8::undefined(&mut scope).into();

        let generator = generator_function.call(&mut scope, this, &[]);
        let generator = v8::Local::<v8::Object>::try_from(generator.unwrap()).unwrap();
        let generator = v8::Global::new(&mut scope, generator);

        state.generator = Some(generator);
    }

    pub fn initialize_clip(&mut self, script: &String) {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

        let future = self.initialize_clip_script(script);

        runtime.block_on(future);
    }

    pub fn advance(&mut self) {
        let state = self.state.lock().unwrap();

        let mut scope = self.js_runtime.handle_scope();

        let generator = state.generator.clone();

        drop(state);

        let generator: v8::Local<v8::Object> = v8::Local::new(&mut scope, generator.unwrap());

        let next_key = v8::String::new(&mut scope, "next").unwrap().into();

        let next_value = generator.get(&mut scope, next_key).unwrap();

        let next = v8::Local::<v8::Function>::try_from(next_value).unwrap();

        let this = v8::Local::<v8::Value>::try_from(generator).unwrap();

        next.call(&mut scope, this, &[]);
    }

    pub fn get_render_data(&self) -> (Vec<u32>, Vec<f32>) {
        let state = self.state.lock().unwrap();

        return (state.indices.clone(), state.vertices.clone());
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
fn op_clip(state: &mut OpState, #[global] value: v8::Global<v8::Function>) -> Result<(), AnyError> {
    let clip_state_mutex = state.borrow_mut::<Arc<Mutex<ClipRuntimeState>>>();
    let mut clip_state = clip_state_mutex.lock().unwrap();

    clip_state.generator_function = Some(value);

    Ok(())
}

#[op2]
fn op_add_element(state: &mut OpState, scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> Result<(), AnyError> {
    let clip_state_mutex = state.borrow_mut::<Arc<Mutex<ClipRuntimeState>>>();
    let mut clip_state = clip_state_mutex.lock().unwrap();

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
