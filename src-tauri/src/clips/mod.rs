use ash::vk;
use image::ImageDecoder;
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{self, read_to_string},
    rc::Rc,
};

use crate::renderer::renderer::{RenderTarget, Renderer};
use crate::renderer::utils::*;
use crate::runtime::ScriptClipRuntime;

pub struct ClipLoader {
    cache: HashMap<String, Rc<RefCell<Clips>>>,
}

impl ClipLoader {
    pub fn new() -> ClipLoader {
        ClipLoader { cache: HashMap::new() }
    }

    pub fn get(&mut self, path: &String, renderer: &Renderer) -> Option<Rc<RefCell<Clips>>> {
        if self.cache.contains_key(path) {
            return Some(self.cache.get(path).unwrap().clone());
        }

        let clip = Rc::new(RefCell::new(self.get_new(path, renderer).unwrap()));

        self.cache.insert(path.clone(), clip.clone());

        Some(clip.clone())
    }

    pub fn get_new(&self, path: &String, renderer: &Renderer) -> Option<Clips> {
        if path.ends_with(".png") {
            let buffer = fs::read(format!("D:/Vector Engine/playground/{}", path)).unwrap();

            let decoder = image::codecs::png::PngDecoder::new(buffer.as_slice()).unwrap();

            let (width, height) = decoder.dimensions();

            let mut bytes = Vec::new();
            bytes.resize((width * height * 4) as usize, 0);

            decoder.read_image(&mut bytes).unwrap();

            return Some(Clips::ImageClip(ImageClip::new(bytes, width, height)));
        }

        Some(Clips::ScriptClip(ScriptClip::new(read_to_string(format!("D:/Vector Engine/playground/{}", path)).unwrap(), renderer)))
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

    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
}

impl ScriptClip {
    pub fn new(script: String, renderer: &Renderer) -> ScriptClip {
        let mut runtime = ScriptClipRuntime::new();

        runtime.initialize_clip(&script);
        runtime.advance();

        let graphics_queue = create_graphics_queue(&renderer.device, renderer.queue_family_index);

        let command_pool = create_command_pool(&renderer.device, renderer.queue_family_index);

        ScriptClip {
            runtime,
            script,
            internal_frame: 0,
            graphics_queue,
            command_pool,
        }
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

    pub fn render(&self, renderer: &mut Renderer, clip_loader: &ClipLoader, width: u32, height: u32) -> RenderTarget {
        let render_target = RenderTarget::new(width, height, renderer);

        let elements = self.runtime.get_elements();

        // let mut ordered_elements = elements.clone();
        // ordered_elements.sort_by(|a, b| a.get_order().partial_cmp(&b.get_order()).unwrap());

        // for element_index in 0..ordered_elements.len() {
        //     let element = &ordered_elements[element_index];

        // match element {
        //     Elements::Rect(rect) => rect.render(
        //         &self.instance,
        //         &self.device,
        //         self.physical_device,
        //         self.target_image_view,
        //         self.command_pool,
        //         self.graphics_queue,
        //         element_index == 0,
        //         element_index == ordered_elements.len() - 1,
        //         self.width,
        //         self.height,
        //     ),
        //     Elements::Ellipse(ellipse) => ellipse.render(
        //         &self.instance,
        //         &self.device,
        //         self.physical_device,
        //         self.target_image_view,
        //         self.command_pool,
        //         self.graphics_queue,
        //         element_index == 0,
        //         element_index == ordered_elements.len() - 1,
        //         self.width,
        //         self.height,
        //     ),
        //     Elements::Clip(clip) => clip.render(
        //         &self.instance,
        //         &self.device,
        //         self.physical_device,
        //         self.target_image_view,
        //         self.command_pool,
        //         self.graphics_queue,
        //         element_index == 0,
        //         element_index == ordered_elements.len() - 1,
        //         self.width,
        //         self.height,
        //         clip_loader,
        //     ),
        // }
        // }

        return render_target;
    }

    pub fn render_to_raw(&self, renderer: &mut Renderer, clip_loader: &ClipLoader, width: u32, height: u32) -> Vec<u8> {
        let render_target = self.render(renderer, clip_loader, width, height);

        render_target.to_raw(&renderer)
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
