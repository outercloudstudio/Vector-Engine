use ash::{
    vk::{self, ShaderModule},
    Device,
};
use image::ImageDecoder;
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{self, read_to_string},
    ptr::copy_nonoverlapping,
    rc::Rc,
    sync::Arc,
};

use crate::renderer::renderer::{RenderTarget, Renderer};
use crate::renderer::utils::*;
use crate::renderer::{
    elements::{Elements, CLIP_DATA_SIZE, CLIP_VERTEX_SIZE, ELLIPSE_DATA_SIZE, ELLIPSE_VERTEX_SIZE, RECT_DATA_SIZE, RECT_VERTEX_SIZE},
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

    ellipse_vertex_shader: ShaderModule,
    ellipse_fragment_shader: ShaderModule,

    ellipse_index_buffer: vk::Buffer,
    ellipse_index_buffer_memory: vk::DeviceMemory,
    ellipse_index_buffer_size: u64,
    ellipse_vertex_buffer: vk::Buffer,
    ellipse_vertex_buffer_memory: vk::DeviceMemory,
    ellipse_vertex_buffer_size: u64,
    ellipse_uniform_buffer: vk::Buffer,
    ellipse_uniform_buffer_memory: vk::DeviceMemory,
    ellipse_uniform_buffer_size: u64,

    clip_vertex_shader: ShaderModule,
    clip_fragment_shader: ShaderModule,

    clip_index_buffer: vk::Buffer,
    clip_index_buffer_memory: vk::DeviceMemory,
    clip_index_buffer_size: u64,
    clip_vertex_buffer: vk::Buffer,
    clip_vertex_buffer_memory: vk::DeviceMemory,
    clip_vertex_buffer_size: u64,
    clip_uniform_buffer: vk::Buffer,
    clip_uniform_buffer_memory: vk::DeviceMemory,
    clip_uniform_buffer_size: u64,
}

impl ScriptClip {
    pub fn new(script: String, renderer: &Renderer) -> ScriptClip {
        let mut runtime = ScriptClipRuntime::new();

        runtime.initialize_clip(&script);
        runtime.advance();

        let graphics_queue = create_graphics_queue(&renderer.device, renderer.queue_family_index);
        let command_pool = create_command_pool(&renderer.device, renderer.queue_family_index);

        let rect_vertex_shader = renderer.create_shader(include_bytes!("./shaders/compiled/rect.vert.spv").to_vec());
        let rect_fragment_shader = renderer.create_shader(include_bytes!("./shaders/compiled/rect.frag.spv").to_vec());

        let (rect_index_buffer, rect_index_buffer_memory, rect_index_buffer_size) = renderer.create_buffer(
            4 * 6,
            vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        let (rect_vertex_buffer, rect_vertex_buffer_memory, rect_vertex_buffer_size) = renderer.create_buffer(
            RECT_VERTEX_SIZE * 4,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        let (rect_uniform_buffer, rect_uniform_buffer_memory, rect_uniform_buffer_size) = renderer.create_buffer(
            RECT_DATA_SIZE,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        let ellipse_vertex_shader = renderer.create_shader(include_bytes!("./shaders/compiled/ellipse.vert.spv").to_vec());
        let ellipse_fragment_shader = renderer.create_shader(include_bytes!("./shaders/compiled/ellipse.frag.spv").to_vec());

        let (ellipse_index_buffer, ellipse_index_buffer_memory, ellipse_index_buffer_size) = renderer.create_buffer(
            4 * 6,
            vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        let (ellipse_vertex_buffer, ellipse_vertex_buffer_memory, ellipse_vertex_buffer_size) = renderer.create_buffer(
            ELLIPSE_VERTEX_SIZE * 4,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        let (ellipse_uniform_buffer, ellipse_uniform_buffer_memory, ellipse_uniform_buffer_size) = renderer.create_buffer(
            ELLIPSE_DATA_SIZE,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        let clip_vertex_shader = renderer.create_shader(include_bytes!("./shaders/compiled/clip.vert.spv").to_vec());
        let clip_fragment_shader = renderer.create_shader(include_bytes!("./shaders/compiled/clip.frag.spv").to_vec());

        let (clip_index_buffer, clip_index_buffer_memory, clip_index_buffer_size) = renderer.create_buffer(
            4 * 6,
            vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        let (clip_vertex_buffer, clip_vertex_buffer_memory, clip_vertex_buffer_size) = renderer.create_buffer(
            CLIP_VERTEX_SIZE * 4,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        let (clip_uniform_buffer, clip_uniform_buffer_memory, clip_uniform_buffer_size) = renderer.create_buffer(
            CLIP_DATA_SIZE,
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

            ellipse_vertex_shader,
            ellipse_fragment_shader,

            ellipse_index_buffer,
            ellipse_index_buffer_memory,
            ellipse_index_buffer_size,
            ellipse_vertex_buffer,
            ellipse_vertex_buffer_memory,
            ellipse_vertex_buffer_size,
            ellipse_uniform_buffer,
            ellipse_uniform_buffer_memory,
            ellipse_uniform_buffer_size,

            clip_vertex_shader,
            clip_fragment_shader,

            clip_index_buffer,
            clip_index_buffer_memory,
            clip_index_buffer_size,
            clip_vertex_buffer,
            clip_vertex_buffer_memory,
            clip_vertex_buffer_size,
            clip_uniform_buffer,
            clip_uniform_buffer_memory,
            clip_uniform_buffer_size,
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

    pub fn render(&self, renderer: &Renderer, clip_loader: &mut ClipLoader, width: u32, height: u32, mode: RenderMode) -> RenderTarget {
        let render_target = RenderTarget::new(width, height, renderer, mode);

        let render_pass = renderer.create_render_pass(mode);

        let frame_buffer = renderer.create_framebuffer(&render_target, render_pass, width, height);

        let viewport = create_viewport(width, height);
        let scissor = create_scissor(width, height);

        let elements = self.runtime.get_elements();

        let mut ordered_elements = elements.clone();
        ordered_elements.sort_by(|a, b| a.get_order().partial_cmp(&b.get_order()).unwrap());

        for element_index in 0..ordered_elements.len() {
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
                Elements::Ellipse(ellipse) => ellipse.render(
                    renderer,
                    self.graphics_queue,
                    render_pass,
                    self.command_pool,
                    frame_buffer,
                    self.ellipse_vertex_shader,
                    self.ellipse_fragment_shader,
                    self.ellipse_index_buffer,
                    self.ellipse_index_buffer_memory,
                    self.ellipse_index_buffer_size,
                    self.ellipse_vertex_buffer,
                    self.ellipse_vertex_buffer_memory,
                    self.ellipse_vertex_buffer_size,
                    self.ellipse_uniform_buffer,
                    self.ellipse_uniform_buffer_memory,
                    self.ellipse_uniform_buffer_size,
                    viewport,
                    scissor,
                    width,
                    height,
                    mode,
                ),
                Elements::Clip(clip) => clip.render(
                    renderer,
                    self.graphics_queue,
                    render_pass,
                    self.command_pool,
                    frame_buffer,
                    self.clip_vertex_shader,
                    self.clip_fragment_shader,
                    self.clip_index_buffer,
                    self.clip_index_buffer_memory,
                    self.clip_index_buffer_size,
                    self.clip_vertex_buffer,
                    self.clip_vertex_buffer_memory,
                    self.clip_vertex_buffer_size,
                    self.clip_uniform_buffer,
                    self.clip_uniform_buffer_memory,
                    self.clip_uniform_buffer_size,
                    viewport,
                    scissor,
                    width,
                    height,
                    clip_loader,
                    mode,
                ),
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
        let graphics_queue = create_graphics_queue(&renderer.device, renderer.queue_family_index);
        let command_pool = create_command_pool(&renderer.device, renderer.queue_family_index);

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
