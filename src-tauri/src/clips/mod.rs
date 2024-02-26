use crate::{runtime::ScriptClipRuntime, Project};

pub trait Clip {
    fn set_frame(&mut self, frame: u32);
    fn render(&self, project: &Project) -> Vec<u8>;
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

    fn render(&self, project: &Project) -> Vec<u8> {
        let (indices, vertices) = self.runtime.get_render_data();

        if indices.len() == 0 {
            return Vec::new();
        }

        let bytes = project.renderer.render(vertices, indices);

        let mut encoded_bytes: Vec<u8> = vec![];

        let mut encoder = png::Encoder::new(&mut encoded_bytes, 512, 512);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&bytes).unwrap();
        writer.finish().unwrap();

        return encoded_bytes;
    }
}
