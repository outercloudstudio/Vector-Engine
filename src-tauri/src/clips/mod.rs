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
        let before_advance = Instant::now();

        self.runtime.advance();

        info!("Advanced in {}ms", before_advance.elapsed().as_millis());
    }

    fn render(&self, renderer: &mut Renderer) -> Vec<u8> {
        let elements = self.runtime.get_elements();

        if elements.len() == 0 {
            return Vec::new();
        }

        let before_render = Instant::now();

        let bytes = renderer.render(elements);

        info!("Rendered in {}ms", before_render.elapsed().as_millis());

        return bytes;

        // let before_encode = Instant::now();

        // let mut encoded_bytes: Vec<u8> = vec![];

        // let mut encoder = png::Encoder::new(&mut encoded_bytes, 480, 270);
        // encoder.set_color(png::ColorType::Rgba);
        // encoder.set_depth(png::BitDepth::Eight);

        // let mut writer = encoder.write_header().unwrap();
        // writer.write_image_data(&bytes).unwrap();
        // writer.finish().unwrap();

        // info!("Encoded in {}ms", before_encode.elapsed().as_millis());

        // return encoded_bytes;
    }
}
