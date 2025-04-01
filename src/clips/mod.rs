use ash::{
    vk::{self, ShaderModule},
    Device,
};
use image::ImageDecoder;
use log::info;
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{self, read_to_string},
    ptr::copy_nonoverlapping,
    rc::Rc,
    sync::Arc,
    time::Instant,
};

use crate::renderer::elements::Elements;
use crate::renderer::renderer::{RenderTarget, Renderer};
use crate::renderer::{
    elements::{ElementRenderContext, PatchRenderContext},
    utils::*,
};
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

            return Some(Clips::ImageClip(ImageClip::new(bytes, width, height, renderer)));
        }

        Some(Clips::ScriptClip(ScriptClip::new(read_to_string(format!("D:/Vector Engine/playground/{}", path)).unwrap(), renderer)))
    }

    pub fn invalidate(&mut self, path: &String) {
        if !self.cache.contains_key(path) {
            return;
        }

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

    element_render_context: ElementRenderContext,
}

impl ScriptClip {
    pub fn new(script: String, renderer: &Renderer) -> ScriptClip {
        let runtime = Self::initialize_runtime(&script);

        let element_render_context = ElementRenderContext::new(renderer);

        ScriptClip {
            runtime,
            script,
            internal_frame: 0,

            element_render_context,
        }
    }

    fn initialize_runtime(script: &String) -> ScriptClipRuntime {
        let mut runtime = ScriptClipRuntime::new();

        let initialized = runtime.initialize_clip(script);

        if initialized.is_ok() {
            runtime.advance();
        }

        return runtime;
    }

    pub fn set_frame(&mut self, frame: u32) {
        if self.internal_frame == frame {
            return;
        }

        if self.internal_frame > frame {
            self.internal_frame = 0;

            let intialized = self.runtime.initialize_clip(&self.script);

            if intialized.is_ok() {
                self.runtime.advance();
            }
        }

        for _ in (self.internal_frame + 1)..=frame {
            self.runtime.advance();
        }

        self.internal_frame = frame;
    }

    pub fn render(&self, renderer: &Renderer, clip_loader: &mut ClipLoader, width: u32, height: u32, render_target: &RenderTarget) {
        let elements = self.runtime.get_elements();

        let mut ordered_elements = elements.clone();
        ordered_elements.sort_by(|a, b| a.get_order().partial_cmp(&b.get_order()).unwrap());

        for element_index in 0..ordered_elements.len() {
            let element = &ordered_elements[element_index];

            match element {
                Elements::Rect(rect) => rect.render(renderer, &self.element_render_context, &render_target),
                Elements::Clip(clip) => clip.render(renderer, &self.element_render_context, &render_target, clip_loader),
            }
        }
    }

    pub fn render_to_raw(&self, renderer: &mut Renderer, clip_loader: &mut ClipLoader, width: u32, height: u32) -> Vec<u8> {
        let render_target = RenderTarget::new(width, height, renderer);

        self.render(renderer, clip_loader, width, height, &render_target);

        let bytes = render_target.to_raw(&renderer);

        return bytes;
    }
}

pub struct ImageClip {
    pub width: u32,
    pub height: u32,

    bytes: Vec<u8>,
    render_target: Arc<RenderTarget>,
}

impl ImageClip {
    pub fn new(bytes: Vec<u8>, width: u32, height: u32, renderer: &Renderer) -> ImageClip {
        let graphics_queue = renderer.create_graphics_queue();
        let command_pool = renderer.create_command_pool();

        let render_target = RenderTarget::new(width, height, renderer);
        let image_data = render_target.image_data.as_ref().unwrap();

        let (staging_buffer, staging_buffer_memory, staging_buffer_size) = renderer.create_buffer(
            bytes.len() as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        let ptr = renderer.start_copy_data_to_buffer(staging_buffer_size, staging_buffer_memory);

        unsafe {
            copy_nonoverlapping(bytes.as_ptr(), ptr.cast(), bytes.len());
        }

        renderer.end_copy_data_to_buffer(staging_buffer_memory);

        transition_image_layout(
            &renderer.device,
            image_data.image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            command_pool,
            graphics_queue,
        );

        copy_buffer_to_image(&renderer.device, staging_buffer, image_data.image, width, height, command_pool, graphics_queue);

        transition_image_layout(
            &renderer.device,
            image_data.image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            command_pool,
            graphics_queue,
        );

        unsafe {
            renderer.device.destroy_buffer(staging_buffer, None);
            renderer.device.free_memory(staging_buffer_memory, None);

            renderer.device.destroy_command_pool(command_pool, None);
        }

        ImageClip {
            bytes,
            width,
            height,
            render_target: Arc::new(render_target),
        }
    }

    pub fn set_frame(&mut self, frame: u32) {}

    pub fn render(&self, renderer: &Renderer, render_target: &RenderTarget) {}

    pub fn render_to_raw(&self, renderer: &Renderer, clip_loader: &ClipLoader) -> Vec<u8> {
        self.bytes.clone()
    }
}
