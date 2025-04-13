use ash::{
    vk::{self, ShaderModule},
    Device,
};
use cgmath::{vec2, Vector2};
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
    internal_render_target: RenderTarget,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ImageVertex {
    pub uv: Vector2<f32>,
}

const UVS: [Vector2<f32>; 4] = [vec2(0.0, 1.0), vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(1.0, 1.0)];

impl ImageClip {
    pub fn new(bytes: Vec<u8>, width: u32, height: u32, renderer: &Renderer) -> ImageClip {
        let internal_render_target = RenderTarget::new(width, height, &renderer);

        let graphics_queue = renderer.create_graphics_queue();
        let command_pool = renderer.create_command_pool();

        let image_data = internal_render_target.image_data.as_ref().unwrap();

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
            internal_render_target,
        }
    }

    pub fn set_frame(&mut self, frame: u32) {}

    pub fn render(&self, renderer: &Renderer, render_target: &RenderTarget) {
        let device = renderer.device.clone();

        let graphics_queue = renderer.create_graphics_queue();
        let command_pool = renderer.create_command_pool();

        let vertex_shader = renderer.create_shader(include_bytes!("../renderer/shaders/compiled/image.vert.spv").to_vec());
        let fragment_shader = renderer.create_shader(include_bytes!("../renderer/shaders/compiled/image.frag.spv").to_vec());

        let (index_buffer, index_buffer_memory, index_buffer_size) = renderer.create_buffer(
            4 * 6,
            vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        let (vertex_buffer, vertex_buffer_memory, vertex_buffer_size) = renderer.create_buffer(
            size_of::<ImageVertex>() as u64 * 4,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        let index_ptr = renderer.start_copy_data_to_buffer(index_buffer_size, index_buffer_memory);

        unsafe {
            copy_nonoverlapping(vec![0, 1, 2, 2, 3, 0].as_ptr(), index_ptr.cast(), 6);
        }

        renderer.end_copy_data_to_buffer(index_buffer_memory);

        let mut vertices: Vec<ImageVertex> = Vec::new();

        for index in 0..4 {
            vertices.push(ImageVertex { uv: UVS[index] });
        }

        let vertex_ptr = renderer.start_copy_data_to_buffer(vertex_buffer_size, vertex_buffer_memory);

        unsafe {
            copy_nonoverlapping(vertices.as_ptr(), vertex_ptr.cast(), vertices.len());
        }

        renderer.end_copy_data_to_buffer(vertex_buffer_memory);

        let image_view = &self.internal_render_target.image_data.as_ref().unwrap().image_view;

        unsafe {
            let sampler_binding = vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT);

            let descriptor_set_layout_binding = vk::VertexInputBindingDescription::default()
                .binding(0)
                .stride(size_of::<ImageVertex>() as u32)
                .input_rate(vk::VertexInputRate::VERTEX);

            let uv_attribute_description = vk::VertexInputAttributeDescription::default().binding(0).location(0).format(vk::Format::R32G32_SFLOAT).offset(0);

            let descriptor_set_layout = renderer.create_descriptor_set_layout(vec![sampler_binding]);
            let descriptor_set_layout_bindings = descriptor_set_layout_binding;
            let attribute_descriptions = vec![uv_attribute_description];

            let (graphics_pipeline, graphics_pipeline_layout) = renderer.create_graphics_pipeline(
                vertex_shader,
                fragment_shader,
                render_target.viewport,
                render_target.scissor,
                render_target.render_pass,
                descriptor_set_layout,
                descriptor_set_layout_bindings,
                &attribute_descriptions,
            );

            let descriptor_pool = renderer.create_descriptor_pool(vec![vk::DescriptorPoolSize::default().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1)]);

            let sampler = renderer.create_sampler();

            let layouts = vec![descriptor_set_layout; 1];
            let info = vk::DescriptorSetAllocateInfo::default().descriptor_pool(descriptor_pool).set_layouts(&layouts);

            let descriptor_sets = device.allocate_descriptor_sets(&info).unwrap();

            let info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(*image_view)
                .sampler(sampler);

            let image_info = &[info];
            let sampler_write = vk::WriteDescriptorSet::default()
                .dst_set(descriptor_sets[0])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(image_info);

            device.update_descriptor_sets(&[sampler_write], &[] as &[vk::CopyDescriptorSet]);

            let command_buffer = renderer.create_command_buffer(command_pool);

            renderer.begin_render_pass(
                render_target.render_pass,
                render_target.frame_buffer,
                command_buffer,
                graphics_pipeline,
                render_target.viewport,
                render_target.scissor,
                render_target.width,
                render_target.height,
            );

            renderer.device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer], &[0]);
            renderer.device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT32);
            renderer
                .device
                .cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline_layout, 0, &descriptor_sets, &[]);

            renderer.device.cmd_draw_indexed(command_buffer, 6, 1, 0, 0, 1);

            renderer.end_render_pass(command_buffer, graphics_queue);
            renderer.execute_render_pass(command_buffer, graphics_queue);

            renderer.device.destroy_sampler(sampler, None);

            renderer.device.destroy_descriptor_pool(descriptor_pool, None);

            renderer.device.destroy_pipeline(graphics_pipeline, None);
            renderer.device.destroy_pipeline_layout(graphics_pipeline_layout, None);

            renderer.device.destroy_descriptor_set_layout(descriptor_set_layout, None);

            renderer.device.destroy_command_pool(command_pool, None);
        }
    }

    pub fn render_to_raw(&self, renderer: &Renderer, clip_loader: &ClipLoader) -> Vec<u8> {
        self.bytes.clone()
    }
}
