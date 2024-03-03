use std::time::Instant;

use log::info;

use crate::{
    renderer::{
        elements::{Elements, Rect},
        Renderer,
    },
    runtime::ScriptClipRuntime,
};

pub enum Clips {
    ScriptClip(ScriptClip),
}

pub trait Clip {
    fn set_frame(&mut self, frame: u32);
    fn render(&self, project: &mut Renderer) -> Vec<u8>;
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

impl Clip for ScriptClip {
    fn set_frame(&mut self, frame: u32) {
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

    fn render(&self, renderer: &mut Renderer) -> Vec<u8> {
        let elements = self.runtime.get_elements();

        if elements.len() == 0 {
            return Vec::new();
        }

        let bytes = renderer.render(elements);

        return bytes;
    }
}
