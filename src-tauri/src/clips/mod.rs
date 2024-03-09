use ash::vk::{self, ShaderModule};
use cgmath::{vec2, Vector2, Vector4};
use image::ImageDecoder;
use log::warn;
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{self, read_to_string},
    mem::align_of,
    ptr::copy_nonoverlapping,
    rc::Rc,
};

use crate::renderer::elements::{Elements, Rect, RectData, RectVertex, RECT_DATA_SIZE, RECT_VERTEX_SIZE};
use crate::renderer::renderer::{RenderTarget, Renderer};
use crate::renderer::utils::*;
use crate::runtime::ScriptClipRuntime;

const UVS: [Vector2<f32>; 4] = [vec2(0.0, 1.0), vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(1.0, 1.0)];

fn rotate(point: Vector2<f32>, origin: Vector2<f32>, angle: f32) -> Vector2<f32> {
    let offset = vec2(point.x - origin.x, point.y - origin.y);

    let rotated = vec2(offset.x * angle.cos() - offset.y * angle.sin(), offset.y * angle.cos() + offset.x * angle.sin());

    vec2(origin.x + rotated.x, origin.y + rotated.y)
}

fn divide(a: Vector2<f32>, b: Vector2<f32>) -> Vector2<f32> {
    vec2(a.x / b.x, a.y / b.y)
}

fn flip_vertically(a: Vector2<f32>) -> Vector2<f32> {
    vec2(a.x, -a.y)
}

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

    rect_vertex_shader: ShaderModule,
    rect_fragment_shader: ShaderModule,

    rect_index_buffer: vk::Buffer,
    rect_index_buffer_memory: vk::DeviceMemory,
    rect_vertex_buffer: vk::Buffer,
    rect_vertex_buffer_memory: vk::DeviceMemory,
    rect_uniform_buffer: vk::Buffer,
    rect_uniform_buffer_memory: vk::DeviceMemory,
    render_pass: vk::RenderPass,
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

        let (rect_index_buffer, rect_index_buffer_memory) = renderer.create_buffer(
            4 * 6,
            vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        let (rect_vertex_buffer, rect_vertex_buffer_memory) = renderer.create_buffer(
            RECT_VERTEX_SIZE * 4,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        let (rect_uniform_buffer, rect_uniform_buffer_memory) = renderer.create_buffer(
            RECT_DATA_SIZE,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        let render_pass = renderer.create_render_pass();

        ScriptClip {
            runtime,
            script,
            internal_frame: 0,

            graphics_queue,
            command_pool,

            rect_vertex_shader,
            rect_fragment_shader,

            rect_index_buffer,
            rect_index_buffer_memory,
            rect_vertex_buffer,
            rect_vertex_buffer_memory,
            rect_uniform_buffer,
            rect_uniform_buffer_memory,

            render_pass,
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

        let frame_buffer = renderer.create_framebuffer(&render_target, self.render_pass, width, height);

        let viewport = create_viewport(width, height);
        let scissor = create_scissor(width, height);

        let elements = self.runtime.get_elements();

        let mut ordered_elements = elements.clone();
        ordered_elements.sort_by(|a, b| a.get_order().partial_cmp(&b.get_order()).unwrap());

        for element_index in 0..ordered_elements.len() {
            let element = &ordered_elements[element_index];

            match element {
                Elements::Rect(rect) => {
                    warn!("Rendering rect");

                    let index_ptr = renderer.start_copy_data_to_buffer(4 * 6, self.rect_index_buffer_memory);

                    unsafe {
                        copy_nonoverlapping(&vec![0, 1, 2, 2, 3, 0].as_ptr(), index_ptr.cast(), 6);
                    }

                    renderer.end_copy_data_to_buffer(self.rect_index_buffer_memory);

                    let normalize_scale = vec2(1920.0 / 2.0, 1080.0 / 2.0);

                    let offsetted_x = rect.position.x - rect.origin.x * rect.size.x;
                    let offsetted_y = rect.position.y - rect.origin.y * rect.size.y;

                    let mut vertices = vec![
                        vec2(offsetted_x, offsetted_y),
                        vec2(offsetted_x, offsetted_y + rect.size.y),
                        vec2(offsetted_x + rect.size.x, offsetted_y + rect.size.y),
                        vec2(offsetted_x + rect.size.x, offsetted_y),
                    ];

                    for vertex_index in 0..vertices.len() {
                        vertices[vertex_index] = flip_vertically(divide(rotate(vertices[vertex_index], rect.position, rect.rotation), normalize_scale))
                    }

                    let vertex_ptr = renderer.start_copy_data_to_buffer(RECT_VERTEX_SIZE * 4, self.rect_vertex_buffer_memory);

                    unsafe {
                        copy_nonoverlapping(&vertices.as_ptr(), vertex_ptr.cast(), 6);
                    }

                    renderer.end_copy_data_to_buffer(self.rect_vertex_buffer_memory);

                    let uniform_ptr = renderer.start_copy_data_to_buffer(RECT_DATA_SIZE, self.rect_uniform_buffer_memory);

                    unsafe {
                        let mut align = ash::util::Align::new(uniform_ptr, align_of::<f32>() as u64, RECT_DATA_SIZE);
                        align.copy_from_slice(&[RectData {
                            color: rect.color,
                            radius: rect.radius,
                            size: rect.size,
                        }]);
                    }

                    renderer.end_copy_data_to_buffer(self.rect_uniform_buffer_memory);

                    let descriptor_set_layouts = Rect::create_descriptor_set_layout(&renderer);
                    let descriptor_set_layout_bindings = RectVertex::get_descriptor_set_layout_binding();
                    let attribute_descriptions = RectVertex::get_attribute_descriptions();

                    let (graphics_pipeline, graphics_pipeline_layout) = renderer.create_graphics_pipeline(
                        self.rect_vertex_shader,
                        self.rect_fragment_shader,
                        viewport,
                        scissor,
                        self.render_pass,
                        descriptor_set_layouts,
                        descriptor_set_layout_bindings,
                        &attribute_descriptions,
                    );

                    let descriptor_pools = Rect::create_descriptor_pool(&renderer);
                    let descriptor_sets = Rect::create_descriptor_sets(&renderer, descriptor_set_layouts, descriptor_pools, self.rect_uniform_buffer);

                    let command_buffer = renderer.create_command_buffer(self.command_pool);

                    renderer.begin_render_pass(self.render_pass, frame_buffer, command_buffer, graphics_pipeline, viewport, scissor, width, height);

                    unsafe {
                        renderer.device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.rect_vertex_buffer], &[0]);
                        renderer.device.cmd_bind_index_buffer(command_buffer, self.rect_index_buffer, 0, vk::IndexType::UINT32);
                        renderer
                            .device
                            .cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline_layout, 0, &descriptor_sets, &[]);
                        renderer.device.cmd_draw_indexed(command_buffer, 6, 1, 0, 0, 1);
                    }

                    renderer.end_render_pass(command_buffer, self.graphics_queue);

                    unsafe {
                        renderer.device.destroy_descriptor_pool(descriptor_pools, None);

                        renderer.device.destroy_pipeline(graphics_pipeline, None);
                        renderer.device.destroy_pipeline_layout(graphics_pipeline_layout, None);

                        renderer.device.destroy_descriptor_set_layout(descriptor_set_layouts, None);
                    }
                }
                _ => {} // Elements::Ellipse(ellipse) => ellipse.render(
                        //     &self.instance,
                        //     &self.device,
                        //     self.physical_device,
                        //     self.target_image_view,
                        //     self.command_pool,
                        //     self.graphics_queue,
                        //     element_index == 0,
                        //     element_index == ordered_elements.len() - 1,
                        //     self.width,
                        //     self.height,
                        // ),
                        // Elements::Clip(clip) => clip.render(
                        //     &self.instance,
                        //     &self.device,
                        //     self.physical_device,
                        //     self.target_image_view,
                        //     self.command_pool,
                        //     self.graphics_queue,
                        //     element_index == 0,
                        //     element_index == ordered_elements.len() - 1,
                        //     self.width,
                        //     self.height,
                        //     clip_loader,
                        // ),
            }
        }

        unsafe {
            renderer.device.destroy_framebuffer(frame_buffer, None);
        }

        return render_target;
    }

    pub fn render_to_raw(&self, renderer: &mut Renderer, clip_loader: &ClipLoader, width: u32, height: u32) -> Vec<u8> {
        let render_target = self.render(renderer, clip_loader, width, height);

        render_target.to_raw(&renderer)
    }

    pub fn destroy(self, renderer: &Renderer) {
        unsafe {
            renderer.device.destroy_shader_module(self.rect_vertex_shader, None);
            renderer.device.destroy_shader_module(self.rect_fragment_shader, None);

            renderer.device.destroy_render_pass(self.render_pass, None);

            renderer.device.destroy_buffer(self.rect_index_buffer, None);
            renderer.device.free_memory(self.rect_index_buffer_memory, None);

            renderer.device.destroy_buffer(self.rect_vertex_buffer, None);
            renderer.device.free_memory(self.rect_vertex_buffer_memory, None);

            renderer.device.destroy_buffer(self.rect_uniform_buffer, None);
            renderer.device.free_memory(self.rect_uniform_buffer_memory, None);
        }
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
