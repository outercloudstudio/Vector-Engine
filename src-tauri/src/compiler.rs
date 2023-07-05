extern crate swc_common;
extern crate swc_ecma_parser;
use swc::config::{Config, IsModule, JscConfig, Options};
use swc_common::sync::Lrc;
use swc_common::{
    errors::{ColorConfig, Handler},
    FileName, SourceMap,
};
use swc_common::{Globals, Mark, GLOBALS};
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::Syntax;

fn main() {
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));
    let compiler = swc::Compiler::new(cm.clone());
    let globals = Globals::default();

    GLOBALS.set(&globals, || {
        let fm = cm.new_source_file(
            FileName::Custom("test.ts".into()),
            "function foo(a: number) { return '' }".into(),
        );

        match compiler.parse_js(
            fm,
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

                return;
            }
        };
    })
}
