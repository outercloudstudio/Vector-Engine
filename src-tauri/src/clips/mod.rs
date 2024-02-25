use crate::Project;

pub trait Clip: Send {
    fn set_frame(&mut self, frame: u32);
    fn render(&self, project: &mut Project) -> Vec<u8>;
}

pub struct ScriptClip {
    script: String,
}

impl ScriptClip {
    pub fn new(script: String) -> ScriptClip {
        ScriptClip { script }
    }
}

impl Clip for ScriptClip {
    fn set_frame(&mut self, _frame: u32) {}

    fn render(&self, project: &mut Project) -> Vec<u8> {
        let vertex_data = project.runtime.execute_clip(&self.script);

        let bytes = project.renderer.render(vertex_data);

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
