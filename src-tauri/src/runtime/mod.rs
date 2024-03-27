use anyhow::Result;
use cgmath::vec2;
use cgmath::vec4;
use cgmath::Vector2;
use cgmath::Vector4;
use deno_ast::MediaType;
use deno_ast::ModuleSpecifier;
use deno_ast::ParseParams;
use deno_ast::SourceTextInfo;
use deno_core::error::AnyError;
use deno_core::futures::FutureExt;
use deno_core::op2;
use deno_core::v8;
use deno_core::v8::GetPropertyNamesArgs;
use deno_core::v8::Object;
use deno_core::Extension;
use deno_core::Op;
use deno_core::{FastString, OpState};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

use crate::renderer::elements::FontAtlas;
use crate::renderer::elements::Text;
use crate::renderer::elements::{Clip, Elements, Ellipse, Rect};

struct ClipRuntimeState {
    elements: Vec<Elements>,
    contexts: Vec<v8::Global<v8::Object>>,
}

pub struct ScriptClipRuntime {
    js_runtime: deno_core::JsRuntime,
    state: Arc<Mutex<ClipRuntimeState>>,
}

impl ScriptClipRuntime {
    pub fn new() -> ScriptClipRuntime {
        let state = Arc::new(Mutex::new(ClipRuntimeState {
            elements: Vec::new(),
            contexts: Vec::new(),
        }));

        let state_arc = state.clone();

        let runtime_extension = Extension::builder("runtime_extension")
            .ops(vec![op_reset_frame::DECL, op_add_frame_element::DECL, op_add_context::DECL])
            .state(|extension_state| {
                extension_state.put::<Arc<Mutex<ClipRuntimeState>>>(state_arc);
            })
            .build();

        let js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
            module_loader: Some(Rc::new(TsModuleLoader)),
            extensions: vec![runtime_extension],
            ..Default::default()
        });

        ScriptClipRuntime { js_runtime, state }
    }

    pub fn initialize_clip(&mut self, script: &String) -> Result<()> {
        let mut state = self.state.lock().unwrap();

        state.elements = Vec::new();
        state.contexts = Vec::new();

        drop(state);

        self.js_runtime.clear_modules();

        let transpiled = transpile_ts(format!(";(globalThis => {{{}}})(globalThis)", String::from(include_str!("./runtime.ts"))))?;
        self.js_runtime.execute_script("vector-engine/runtime.ts", deno_core::FastString::from(transpiled)).unwrap();

        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

        let transpiled = transpile_ts(script.clone())?;
        let clip_module = runtime
            .block_on(self.js_runtime.load_main_module(
                &ModuleSpecifier::from_file_path("D:/Vector Engine/playground/project.ts").unwrap(),
                Some(deno_core::FastString::from(transpiled)),
            ))
            .unwrap();

        let _ = self.js_runtime.mod_evaluate(clip_module);

        runtime.block_on(self.js_runtime.run_event_loop(false)).unwrap();

        Ok(())
    }

    pub fn advance(&mut self) {
        let mut state = self.state.lock().unwrap();

        state.elements = Vec::new();

        drop(state);

        self.advance_contexts();

        self.update_frame();
    }

    fn handle_context(&mut self, context: v8::Global<v8::Object>) {
        let mut scope = self.js_runtime.handle_scope();

        let generator = v8::Local::new(&mut scope, context);

        let key = v8::String::new(&mut scope, "next").unwrap();

        let next = generator.get(&mut scope, key.into()).unwrap();

        let next = v8::Local::<v8::Function>::try_from(next).unwrap();

        let result = next.call(&mut scope, generator.into(), &[]);

        if result.is_none() {
            return;
        }

        let result = result.unwrap();

        let result = v8::Local::<v8::Object>::try_from(result).unwrap();

        let key = v8::String::new(&mut scope, "value").unwrap();

        let result = result.get(&mut scope, key.into()).unwrap();

        if result.is_generator_object() {
            let mut state = self.state.lock().unwrap();

            let result = v8::Local::<v8::Object>::try_from(result).unwrap();

            let result = v8::Global::new(&mut scope, result);

            state.contexts.push(result.clone());

            drop(state);
            drop(scope);

            self.handle_context(result);
        }
    }

    fn advance_contexts(&mut self) {
        let state = self.state.lock().unwrap();

        let contexts = state.contexts.clone();

        drop(state);

        for context in contexts {
            self.handle_context(context);
        }
    }

    fn update_frame(&mut self) {
        let binding = self.js_runtime.main_context();

        let context = binding.open(&mut self.js_runtime.v8_isolate());

        let mut scope = self.js_runtime.handle_scope();

        let global = context.global(&mut scope);

        let this = v8::undefined(&mut scope);

        let update_frame_key = v8::String::new(&mut scope, "_updateFrame").unwrap();

        let update_frame = global.get(&mut scope, update_frame_key.into()).unwrap();

        let update_frame = v8::Local::<v8::Function>::try_from(update_frame).unwrap();

        update_frame.call(&mut scope, this.into(), &[]);
    }

    pub fn get_elements(&self) -> Vec<Elements> {
        let state = self.state.lock().unwrap();

        return state.elements.clone();
    }
}

fn transpile_ts(code: String) -> Result<String> {
    let parsed = deno_ast::parse_module(ParseParams {
        specifier: String::from(""),
        text_info: SourceTextInfo::from_string(code),
        media_type: MediaType::TypeScript,
        capture_tokens: false,
        scope_analysis: false,
        maybe_syntax: None,
    })?;

    Ok(parsed.transpile(&Default::default()).unwrap().text)
}

pub fn deserialize_number(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> f32 {
    v8::Local::<v8::Number>::try_from(value).unwrap().value() as f32
}

pub fn deserialize_string(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> String {
    value.to_rust_string_lossy(scope)
}

pub fn deserialize_vector2(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> Vector2<f32> {
    let object = v8::Local::<v8::Object>::try_from(value).unwrap();

    let x_key = v8::String::new(scope, "x").unwrap().into();
    let x_value = object.get(scope, x_key).unwrap();

    let y_key = v8::String::new(scope, "y").unwrap().into();
    let y_value = object.get(scope, y_key).unwrap();

    vec2(deserialize_number(scope, x_value), deserialize_number(scope, y_value))
}

pub fn deserialize_vector4(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> Vector4<f32> {
    let object = v8::Local::<v8::Object>::try_from(value).unwrap();

    let x_key = v8::String::new(scope, "x").unwrap().into();
    let x_value = object.get(scope, x_key).unwrap();

    let y_key = v8::String::new(scope, "y").unwrap().into();
    let y_value = object.get(scope, y_key).unwrap();

    let z_key = v8::String::new(scope, "z").unwrap().into();
    let z_value = object.get(scope, z_key).unwrap();

    let w_key = v8::String::new(scope, "w").unwrap().into();
    let w_value = object.get(scope, w_key).unwrap();

    vec4(
        deserialize_number(scope, x_value),
        deserialize_number(scope, y_value),
        deserialize_number(scope, z_value),
        deserialize_number(scope, w_value),
    )
}

impl Rect {
    pub fn deserialize(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> Rect {
        let object = v8::Local::<v8::Object>::try_from(value).unwrap();

        let position_key = v8::String::new(scope, "position").unwrap().into();
        let position_value = object.get(scope, position_key).unwrap();

        let origin_key = v8::String::new(scope, "origin").unwrap().into();
        let origin_value = object.get(scope, origin_key).unwrap();

        let size_key = v8::String::new(scope, "size").unwrap().into();
        let size_value = object.get(scope, size_key).unwrap();

        let rotation_key = v8::String::new(scope, "rotation").unwrap().into();
        let rotation_value = object.get(scope, rotation_key).unwrap();

        let color_key = v8::String::new(scope, "color").unwrap().into();
        let color_value = object.get(scope, color_key).unwrap();

        let radius_key = v8::String::new(scope, "radius").unwrap().into();
        let radius_value = object.get(scope, radius_key).unwrap();

        let order_key = v8::String::new(scope, "order").unwrap().into();
        let order_value = object.get(scope, order_key).unwrap();

        Rect {
            position: deserialize_vector2(scope, position_value),
            origin: deserialize_vector2(scope, origin_value),
            size: deserialize_vector2(scope, size_value),
            rotation: deserialize_number(scope, rotation_value),
            color: deserialize_vector4(scope, color_value),
            radius: deserialize_number(scope, radius_value),
            order: deserialize_number(scope, order_value),
        }
    }
}

impl Ellipse {
    pub fn deserialize(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> Ellipse {
        let object = v8::Local::<v8::Object>::try_from(value).unwrap();

        let position_key = v8::String::new(scope, "position").unwrap().into();
        let position_value = object.get(scope, position_key).unwrap();

        let origin_key = v8::String::new(scope, "origin").unwrap().into();
        let origin_value = object.get(scope, origin_key).unwrap();

        let size_key = v8::String::new(scope, "size").unwrap().into();
        let size_value = object.get(scope, size_key).unwrap();

        let color_key = v8::String::new(scope, "color").unwrap().into();
        let color_value = object.get(scope, color_key).unwrap();

        let order_key = v8::String::new(scope, "order").unwrap().into();
        let order_value = object.get(scope, order_key).unwrap();

        Ellipse {
            position: deserialize_vector2(scope, position_value),
            origin: deserialize_vector2(scope, origin_value),
            size: deserialize_vector2(scope, size_value),
            color: deserialize_vector4(scope, color_value),
            order: deserialize_number(scope, order_value),
        }
    }
}

impl Clip {
    pub fn deserialize(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> Clip {
        let object = v8::Local::<v8::Object>::try_from(value).unwrap();

        let position_key = v8::String::new(scope, "position").unwrap().into();
        let position_value = object.get(scope, position_key).unwrap();

        let origin_key = v8::String::new(scope, "origin").unwrap().into();
        let origin_value = object.get(scope, origin_key).unwrap();

        let size_key = v8::String::new(scope, "size").unwrap().into();
        let size_value = object.get(scope, size_key).unwrap();

        let rotation_key = v8::String::new(scope, "rotation").unwrap().into();
        let rotation_value = object.get(scope, rotation_key).unwrap();

        let color_key = v8::String::new(scope, "color").unwrap().into();
        let color_value = object.get(scope, color_key).unwrap();

        let clip_key = v8::String::new(scope, "clip").unwrap().into();
        let clip_value = object.get(scope, clip_key).unwrap();

        let frame_key = v8::String::new(scope, "frame").unwrap().into();
        let frame_value = object.get(scope, frame_key).unwrap();

        let order_key = v8::String::new(scope, "order").unwrap().into();
        let order_value = object.get(scope, order_key).unwrap();

        Clip {
            clip: deserialize_string(scope, clip_value),
            frame: deserialize_number(scope, frame_value) as u32,
            position: deserialize_vector2(scope, position_value),
            origin: deserialize_vector2(scope, origin_value),
            rotation: deserialize_number(scope, rotation_value),
            size: deserialize_vector2(scope, size_value),
            color: deserialize_vector4(scope, color_value),
            order: deserialize_number(scope, order_value),
        }
    }
}

impl FontAtlas {
    pub fn deserialize(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> FontAtlas {
        let object = v8::Local::<v8::Object>::try_from(value).unwrap();

        let path_key = v8::String::new(scope, "path").unwrap().into();
        let path_value = object.get(scope, path_key).unwrap();

        let rows_key = v8::String::new(scope, "rows").unwrap().into();
        let rows_value = object.get(scope, rows_key).unwrap();

        let columns_key = v8::String::new(scope, "columns").unwrap().into();
        let columns_value = object.get(scope, columns_key).unwrap();

        let dropdown_key = v8::String::new(scope, "dropdown").unwrap().into();
        let dropdown_value = object.get(scope, dropdown_key).unwrap();

        let spacing_key = v8::String::new(scope, "spacing").unwrap().into();
        let spacing_value = object.get(scope, spacing_key).unwrap();

        let characters_key = v8::String::new(scope, "characters").unwrap().into();
        let characters_value = object.get(scope, characters_key).unwrap();

        let width_overrides_key = v8::String::new(scope, "widthOverrides").unwrap().into();
        let width_overrides_value = object.get(scope, width_overrides_key).unwrap();

        let mut width_overrides: HashMap<char, f32> = HashMap::new();

        {
            let width_overrides_value = v8::Local::<v8::Object>::try_from(width_overrides_value).unwrap();

            let key_names = width_overrides_value.get_property_names(scope, GetPropertyNamesArgs::default()).unwrap();

            for key_index in 0..key_names.length() {
                let key = key_names.get_index(scope, key_index).unwrap();
                let value = width_overrides_value.get(scope, key).unwrap();

                width_overrides.insert(key.to_rust_string_lossy(scope).chars().next().unwrap(), deserialize_number(scope, value));
            }
        }

        FontAtlas {
            path: deserialize_string(scope, path_value),
            rows: deserialize_number(scope, rows_value) as u32,
            columns: deserialize_number(scope, columns_value) as u32,
            dropdown: deserialize_number(scope, dropdown_value),
            spacing: deserialize_number(scope, spacing_value),
            characters: deserialize_string(scope, characters_value),
            width_overrides,
        }
    }
}

impl Text {
    pub fn deserialize(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> Text {
        let object = v8::Local::<v8::Object>::try_from(value).unwrap();

        let position_key = v8::String::new(scope, "position").unwrap().into();
        let position_value = object.get(scope, position_key).unwrap();

        let origin_key = v8::String::new(scope, "origin").unwrap().into();
        let origin_value = object.get(scope, origin_key).unwrap();

        let size_key = v8::String::new(scope, "size").unwrap().into();
        let size_value = object.get(scope, size_key).unwrap();

        let rotation_key = v8::String::new(scope, "rotation").unwrap().into();
        let rotation_value = object.get(scope, rotation_key).unwrap();

        let color_key = v8::String::new(scope, "color").unwrap().into();
        let color_value = object.get(scope, color_key).unwrap();

        let text_key = v8::String::new(scope, "text").unwrap().into();
        let text_value = object.get(scope, text_key).unwrap();

        let font_key = v8::String::new(scope, "font").unwrap().into();
        let font_value = object.get(scope, font_key).unwrap();

        let order_key = v8::String::new(scope, "order").unwrap().into();
        let order_value = object.get(scope, order_key).unwrap();

        Text {
            text: deserialize_string(scope, text_value),
            font: FontAtlas::deserialize(scope, font_value),
            position: deserialize_vector2(scope, position_value),
            origin: deserialize_vector2(scope, origin_value),
            rotation: deserialize_number(scope, rotation_value),
            size: deserialize_number(scope, size_value),
            color: deserialize_vector4(scope, color_value),
            order: deserialize_number(scope, order_value),
        }
    }
}

#[op2]
fn op_reset_frame(state: &mut OpState, scope: &mut v8::HandleScope) -> Result<(), AnyError> {
    let state_mutex = state.borrow_mut::<Arc<Mutex<ClipRuntimeState>>>();
    let mut state = state_mutex.lock().unwrap();

    state.elements = Vec::new();

    Ok(())
}

#[op2]
fn op_add_frame_element(state: &mut OpState, scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> Result<(), AnyError> {
    let state_mutex = state.borrow_mut::<Arc<Mutex<ClipRuntimeState>>>();
    let mut state = state_mutex.lock().unwrap();

    let object = v8::Local::<v8::Object>::try_from(value).unwrap();

    let type_key = v8::String::new(scope, "type").unwrap().into();
    let type_value = object.get(scope, type_key).unwrap();
    let type_string = v8::Local::<v8::String>::try_from(type_value).unwrap().to_rust_string_lossy(scope);

    if type_string == "Rect" {
        state.elements.push(Elements::Rect(Rect::deserialize(scope, value)));
    }

    if type_string == "Ellipse" {
        state.elements.push(Elements::Ellipse(Ellipse::deserialize(scope, value)));
    }

    if type_string == "Clip" {
        state.elements.push(Elements::Clip(Clip::deserialize(scope, value)));
    }

    if type_string == "Text" {
        state.elements.push(Elements::Text(Text::deserialize(scope, value)));
    }

    Ok(())
}

#[op2]
fn op_add_context(state: &mut OpState, scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> Result<(), AnyError> {
    let state_mutex = state.borrow_mut::<Arc<Mutex<ClipRuntimeState>>>();
    let mut state = state_mutex.lock().unwrap();

    let generator = v8::Local::<v8::Object>::try_from(value).unwrap();

    let generator = v8::Global::new(scope, generator);

    state.contexts.push(generator);

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
            let mut path = module_specifier.to_file_path().unwrap();

            let media_type = MediaType::from_path(&path);
            let (module_type, should_transpile) = match media_type {
                MediaType::JavaScript | MediaType::Mjs | MediaType::Cjs => (deno_core::ModuleType::JavaScript, false),
                MediaType::Jsx => (deno_core::ModuleType::JavaScript, true),
                MediaType::TypeScript | MediaType::Mts | MediaType::Cts | MediaType::Dts | MediaType::Dmts | MediaType::Dcts | MediaType::Tsx => (deno_core::ModuleType::JavaScript, true),
                MediaType::Json => (deno_core::ModuleType::Json, false),
                _ => {
                    path.set_extension("ts");

                    (deno_core::ModuleType::JavaScript, true)
                }
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
