use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{self, read_to_string},
    rc::Rc,
    time::Instant,
};

use image::ImageDecoder;
use log::info;

use crate::{renderer::Renderer, runtime::ScriptClipRuntime};

pub struct ClipLoader {
    cache: HashMap<String, Rc<RefCell<Clips>>>,
}

impl ClipLoader {
    pub fn new() -> ClipLoader {
        ClipLoader { cache: HashMap::new() }
    }

    pub fn get(&mut self, path: &String) -> Option<Rc<RefCell<Clips>>> {
        if self.cache.contains_key(path) {
            return Some(self.cache.get(path).unwrap().clone());
        }

        let clip = Rc::new(RefCell::new(self.get_new(path).unwrap()));

        self.cache.insert(path.clone(), clip.clone());

        Some(clip.clone())
    }

    pub fn get_new(&self, path: &String) -> Option<Clips> {
        if path.ends_with(".png") {
            let buffer = fs::read(format!("D:/Vector Engine/playground/{}", path)).unwrap();

            let decoder = image::codecs::png::PngDecoder::new(buffer.as_slice()).unwrap();

            let (width, height) = decoder.dimensions();

            let mut bytes = Vec::new();
            bytes.resize((width * height * 4) as usize, 0);

            decoder.read_image(&mut bytes).unwrap();

            return Some(Clips::ImageClip(ImageClip::new(bytes, width, height)));
        }

        Some(Clips::ScriptClip(ScriptClip::new(read_to_string(format!("D:/Vector Engine/playground/{}", path)).unwrap())))
    }

    pub fn invalidate(&mut self, path: &String) {
        self.cache.remove(path);
    }
}

pub enum Clips {
    ScriptClip(ScriptClip),
    ImageClip(ImageClip),
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

pub struct ImageClip {
    bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl ImageClip {
    pub fn new(bytes: Vec<u8>, width: u32, height: u32) -> ImageClip {
        ImageClip { bytes, width, height }
    }

    pub fn set_frame(&mut self, frame: u32) {}

    pub fn render(&self, renderer: &mut Renderer, clip_loader: &ClipLoader) -> Vec<u8> {
        self.bytes.clone()
    }
}
