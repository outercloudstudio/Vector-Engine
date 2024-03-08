use std::{
    cell::RefCell,
    collections::HashMap,
    fs::read_to_string,
    sync::{Arc, Mutex},
    time::Instant,
};

use log::info;

use crate::{renderer::Renderer, runtime::ScriptClipRuntime};

pub struct ClipLoader {}

impl ClipLoader {
    pub fn new() -> ClipLoader {
        ClipLoader {}
    }

    pub fn get_new(&self, path: String) -> Option<Clips> {
        Some(Clips::ScriptClip(ScriptClip::new(read_to_string(format!("D:/Vector Engine/playground/{}", path)).unwrap())))
    }
}

pub enum Clips {
    ScriptClip(ScriptClip),
}

pub struct ScriptClip {
    runtime: ScriptClipRuntime,
    script: String,
    internal_frame: u32,
}

impl ScriptClip {
    pub fn new(script: String) -> ScriptClip {
        let mut runtime = ScriptClipRuntime::new();

        runtime.initialize_clip(&script);
        runtime.advance();

        ScriptClip { runtime, script, internal_frame: 0 }
    }
}

impl ScriptClip {
    pub fn set_frame(&mut self, frame: u32) {
        if self.internal_frame == frame {
            return;
        }

        if self.internal_frame > frame {
            self.internal_frame = 0;

            self.runtime.initialize_clip(&self.script);
            self.runtime.advance();
        }

        for _ in (self.internal_frame + 1)..=frame {
            self.runtime.advance();
        }

        self.internal_frame = frame;
    }

    pub fn render(&self, renderer: &mut Renderer, clip_loader: &ClipLoader) -> Vec<u8> {
        let elements = self.runtime.get_elements();

        if elements.len() == 0 {
            return Vec::new();
        }

        let bytes = renderer.render(elements, clip_loader);

        return bytes;
    }
}
