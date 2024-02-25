use deno_ast::MediaType;
use deno_ast::ParseParams;
use deno_ast::SourceTextInfo;
use deno_core::error::AnyError;
use deno_core::futures::FutureExt;
use deno_core::op;
use deno_core::FastString;
use log::info;
use std::env;
use std::path::Path;
use std::rc::Rc;

// deno_core::extension!(runtime_extension, ops = []);

pub struct Runtime {
    runtime: tokio::runtime::Runtime,
}

impl Runtime {
    pub fn create() -> Runtime {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

        Runtime { runtime }
    }

    async fn test_hidden(self: &Runtime) {
        let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
            module_loader: Some(Rc::new(TsModuleLoader)),
            // extensions: vec![runtime_extension::init_ops()],
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
    }

    pub fn test(self: &Runtime) {
        self.runtime.block_on(self.test_hidden());
    }
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
