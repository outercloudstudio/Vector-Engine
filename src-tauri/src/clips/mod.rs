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
}

impl ScriptClip {
    pub fn new(script: String) -> ScriptClip {
        let mut runtime = ScriptClipRuntime::new();

        runtime.initialize_clip(&script);

        ScriptClip { runtime }
    }
}

impl Clip for ScriptClip {
    fn set_frame(&mut self, _frame: u32) {
        self.runtime.advance();
    }

    fn render(&self, renderer: &mut Renderer) -> Vec<u8> {
        let (indices, vertices) = self.runtime.get_render_data();

        if indices.len() == 0 {
            return Vec::new();
        }

        let rect = Rect {
            position: cgmath::vec2(0.0, 0.0),
            size: cgmath::vec2(200.0, 200.0),
        };

        let before_render = Instant::now();

        let bytes = renderer.render(vec![Elements::Rect(rect)]);

        info!("Rendered in {}ms", before_render.elapsed().as_millis());

        let mut encoded_bytes: Vec<u8> = vec![];

        let mut encoder = png::Encoder::new(&mut encoded_bytes, 1920, 1080);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&bytes).unwrap();
        writer.finish().unwrap();

        return encoded_bytes;
    }
}
