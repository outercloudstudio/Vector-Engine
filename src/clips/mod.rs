use image::ImageDecoder;
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{self, read_to_string},
    ptr::copy_nonoverlapping,
    rc::Rc,
    sync::Arc,
};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::{
    vk::{self, ShaderModule},
    Device,
};

use crate::renderer::renderer::{RenderTarget, Renderer};
use crate::renderer::utils::*;
use crate::renderer::{
    elements::{Elements, RECT_DATA_SIZE, UV_VERTEX_SIZE},
    renderer::RenderMode,
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

    device: Device,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,

    rect_vertex_shader: ShaderModule,
    rect_fragment_shader: ShaderModule,

    rect_index_buffer: vk::Buffer,
    rect_index_buffer_memory: vk::DeviceMemory,
    rect_index_buffer_size: u64,
    rect_vertex_buffer: vk::Buffer,
    rect_vertex_buffer_memory: vk::DeviceMemory,
    rect_vertex_buffer_size: u64,
    rect_uniform_buffer: vk::Buffer,
    rect_uniform_buffer_memory: vk::DeviceMemory,
    rect_uniform_buffer_size: u64,
}

impl ScriptClip {
    pub fn new(script: String, renderer: &Renderer) -> ScriptClip {
        let mut runtime = ScriptClipRuntime::new();

        let initialized = runtime.initialize_clip(&script);

        if initialized.is_ok() {
            runtime.advance();
        }

        let graphics_queue = renderer.create_graphics_queue();
        let command_pool = renderer.create_command_pool();

        let rect_vertex_shader = renderer.create_shader(include_bytes!("./shaders/compiled/rect.vert.spv").to_vec());
        let rect_fragment_shader = renderer.create_shader(include_bytes!("./shaders/compiled/rect.frag.spv").to_vec());

        let (rect_index_buffer, rect_index_buffer_memory, rect_index_buffer_size) = renderer.create_buffer(
            4 * 6,
            vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        let (rect_vertex_buffer, rect_vertex_buffer_memory, rect_vertex_buffer_size) = renderer.create_buffer(
            UV_VERTEX_SIZE * 4,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        let (rect_uniform_buffer, rect_uniform_buffer_memory, rect_uniform_buffer_size) = renderer.create_buffer(
            RECT_DATA_SIZE,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        ScriptClip {
            runtime,
            script,
            internal_frame: 0,

            graphics_queue,
            command_pool,
            device: renderer.device.clone(),

            rect_vertex_shader,
            rect_fragment_shader,

            rect_index_buffer,
            rect_index_buffer_memory,
            rect_index_buffer_size,
            rect_vertex_buffer,
            rect_vertex_buffer_memory,
            rect_vertex_buffer_size,
            rect_uniform_buffer,
            rect_uniform_buffer_memory,
            rect_uniform_buffer_size,
        }
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

    pub fn render(&self, renderer: &Renderer, clip_loader: &mut ClipLoader, width: u32, height: u32, mode: RenderMode) -> RenderTarget {
        let elements = self.runtime.get_elements();

        let mut ordered_elements = elements.clone();
        ordered_elements.sort_by(|a, b| a.get_order().partial_cmp(&b.get_order()).unwrap());

        let render_target = RenderTarget::new(width, height, renderer, mode);

        let mut render_pass = renderer.create_render_pass(vk::ImageLayout::UNDEFINED, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let frame_buffer = renderer.create_framebuffer(&render_target, render_pass, width, height);

        let viewport = renderer.create_viewport(width, height);
        let scissor = renderer.create_scissor(width, height);

        for element_index in 0..ordered_elements.len() {
            if element_index == elements.len() - 1 {
                render_pass = renderer.create_render_pass(
                    if element_index == 0 {
                        vk::ImageLayout::UNDEFINED
                    } else {
                        vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
                    },
                    if let RenderMode::Raw = mode {
                        vk::ImageLayout::TRANSFER_SRC_OPTIMAL
                    } else {
                        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
                    },
                )
            }

            let element = &ordered_elements[element_index];

            match element {
                Elements::Rect(rect) => rect.render(
                    renderer,
                    self.graphics_queue,
                    render_pass,
                    self.command_pool,
                    frame_buffer,
                    self.rect_vertex_shader,
                    self.rect_fragment_shader,
                    self.rect_index_buffer,
                    self.rect_index_buffer_memory,
                    self.rect_index_buffer_size,
                    self.rect_vertex_buffer,
                    self.rect_vertex_buffer_memory,
                    self.rect_vertex_buffer_size,
                    self.rect_uniform_buffer,
                    self.rect_uniform_buffer_memory,
                    self.rect_uniform_buffer_size,
                    viewport,
                    scissor,
                    width,
                    height,
                    mode,
                ),
            }

            if element_index == 0 {
                render_pass = renderer.create_render_pass(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            }
        }

        unsafe {
            renderer.device.destroy_framebuffer(frame_buffer, None);

            self.device.destroy_render_pass(render_pass, None);
        }

        return render_target;
    }

    pub fn render_elements(&self, elements: &Vec<Elements>, renderer: &Renderer, width: u32, height: u32, mode: RenderMode) -> RenderTarget {
        let mut ordered_elements = elements.clone();
        ordered_elements.sort_by(|a, b| a.get_order().partial_cmp(&b.get_order()).unwrap());

        let render_target = RenderTarget::new(width, height, renderer, mode);

        let mut render_pass = renderer.create_render_pass(vk::ImageLayout::UNDEFINED, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let frame_buffer = renderer.create_framebuffer(&render_target, render_pass, width, height);

        let viewport = renderer.create_viewport(width, height);
        let scissor = renderer.create_scissor(width, height);

        for element_index in 0..ordered_elements.len() {
            if element_index == elements.len() - 1 {
                render_pass = renderer.create_render_pass(
                    if element_index == 0 {
                        vk::ImageLayout::UNDEFINED
                    } else {
                        vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
                    },
                    if let RenderMode::Raw = mode {
                        vk::ImageLayout::TRANSFER_SRC_OPTIMAL
                    } else {
                        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
                    },
                )
            }

            let element = &ordered_elements[element_index];

            match element {
                Elements::Rect(rect) => rect.render(
                    renderer,
                    self.graphics_queue,
                    render_pass,
                    self.command_pool,
                    frame_buffer,
                    self.rect_vertex_shader,
                    self.rect_fragment_shader,
                    self.rect_index_buffer,
                    self.rect_index_buffer_memory,
                    self.rect_index_buffer_size,
                    self.rect_vertex_buffer,
                    self.rect_vertex_buffer_memory,
                    self.rect_vertex_buffer_size,
                    self.rect_uniform_buffer,
                    self.rect_uniform_buffer_memory,
                    self.rect_uniform_buffer_size,
                    viewport,
                    scissor,
                    width,
                    height,
                    mode,
                ),
                _ => {}
            }

            if element_index == 0 {
                render_pass = renderer.create_render_pass(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            }
        }

        unsafe {
            renderer.device.destroy_framebuffer(frame_buffer, None);

            self.device.destroy_render_pass(render_pass, None);
        }

        return render_target;
    }

    pub fn render_to_raw(&self, renderer: &mut Renderer, clip_loader: &mut ClipLoader, width: u32, height: u32) -> Vec<u8> {
        let render_target = self.render(renderer, clip_loader, width, height, RenderMode::Raw);

        let bytes = render_target.to_raw(&renderer);

        return bytes;
    }
}

impl Drop for ScriptClip {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_shader_module(self.rect_vertex_shader, None);
            self.device.destroy_shader_module(self.rect_fragment_shader, None);

            self.device.destroy_buffer(self.rect_index_buffer, None);
            self.device.free_memory(self.rect_index_buffer_memory, None);

            self.device.destroy_buffer(self.rect_vertex_buffer, None);
            self.device.free_memory(self.rect_vertex_buffer_memory, None);

            self.device.destroy_buffer(self.rect_uniform_buffer, None);
            self.device.free_memory(self.rect_uniform_buffer_memory, None);

            self.device.destroy_command_pool(self.command_pool, None);
        }
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

        let render_target = RenderTarget::new(width, height, renderer, RenderMode::Sample);

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
            render_target.image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            command_pool,
            graphics_queue,
        );

        copy_buffer_to_image(&renderer.device, staging_buffer, render_target.image, width, height, command_pool, graphics_queue);

        transition_image_layout(
            &renderer.device,
            render_target.image,
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

    pub fn render(&self, renderer: &Renderer, clip_loader: &ClipLoader) -> Arc<RenderTarget> {
        self.render_target.clone()
    }

    pub fn render_to_raw(&self, renderer: &Renderer, clip_loader: &ClipLoader) -> Vec<u8> {
        self.bytes.clone()
    }
}
