use ash::vk;
use ash::vk::ShaderModule;
use cgmath::{vec2, Vector2, Vector4};
use log::info;
use std::collections::HashMap;
use std::mem::align_of;
use std::ptr::copy_nonoverlapping;

use super::renderer::RenderMode;
use crate::clips::{ClipLoader, Clips};
use crate::renderer::renderer::Renderer;

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

#[derive(Clone)]
pub enum Elements {
    Rect(Rect),
    Ellipse(Ellipse),
    Clip(Clip),
    Text(Text),
}

impl Elements {
    pub fn get_order(&self) -> f32 {
        match &self {
            Elements::Rect(rect) => rect.order,
            Elements::Ellipse(ellipse) => ellipse.order,
            Elements::Clip(clip) => clip.order,
            Elements::Text(text) => text.order,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UvVertex {
    pub position: Vector2<f32>,
    pub uv: Vector2<f32>,
}
pub const UV_VERTEX_SIZE: u64 = 8 + 8;

impl UvVertex {
    pub fn get_descriptor_set_layout_binding() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(UV_VERTEX_SIZE as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        let position = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(0)
            .build();

        let uv = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(8)
            .build();

        [position, uv]
    }
}

#[derive(Clone)]
pub struct Rect {
    pub position: Vector2<f32>,
    pub origin: Vector2<f32>,
    pub size: Vector2<f32>,
    pub rotation: f32,
    pub color: Vector4<f32>,
    pub radius: f32,
    pub order: f32,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct RectData {
    pub color: Vector4<f32>,
    pub size: Vector2<f32>,
    pub radius: f32,
}

// pub const RECT_DATA_SIZE: u64 = 16 + 4 + 8;
pub const RECT_DATA_SIZE: u64 = 64;

impl RectData {
    pub fn get_descriptor_set_layout_bindings() -> Vec<vk::DescriptorSetLayoutBinding> {
        let layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build();
        vec![layout_binding]
    }
}

impl Rect {
    pub fn render(
        &self,
        renderer: &Renderer,
        graphics_queue: vk::Queue,
        render_pass: vk::RenderPass,
        command_pool: vk::CommandPool,
        frame_buffer: vk::Framebuffer,
        vertex_shader: ShaderModule,
        fragment_shader: ShaderModule,
        index_buffer: vk::Buffer,
        index_buffer_memory: vk::DeviceMemory,
        index_buffer_size: u64,
        vertex_buffer: vk::Buffer,
        vertex_buffer_memory: vk::DeviceMemory,
        vertex_buffer_size: u64,
        uniform_buffer: vk::Buffer,
        uniform_buffer_memory: vk::DeviceMemory,
        uniform_buffer_size: u64,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
        width: u32,
        height: u32,
        mode: RenderMode,
    ) {
        let index_ptr = renderer.start_copy_data_to_buffer(index_buffer_size, index_buffer_memory);

        unsafe {
            copy_nonoverlapping(vec![0, 1, 2, 2, 3, 0].as_ptr(), index_ptr.cast(), 6);
        }

        renderer.end_copy_data_to_buffer(index_buffer_memory);

        let normalize_scale = vec2(1920.0 / 2.0, 1080.0 / 2.0);

        let offsetted_x = self.position.x - self.origin.x * self.size.x;
        let offsetted_y = self.position.y - self.origin.y * self.size.y;

        let mut vertex_positions: Vec<Vector2<f32>> = vec![
            vec2(offsetted_x, offsetted_y),
            vec2(offsetted_x, offsetted_y + self.size.y),
            vec2(offsetted_x + self.size.x, offsetted_y + self.size.y),
            vec2(offsetted_x + self.size.x, offsetted_y),
        ];

        for vertex_position_index in 0..vertex_positions.len() {
            vertex_positions[vertex_position_index] = flip_vertically(divide(rotate(vertex_positions[vertex_position_index], self.position, self.rotation), normalize_scale));
        }

        let mut vertices: Vec<UvVertex> = Vec::new();

        for index in 0..vertex_positions.len() {
            vertices.push(UvVertex {
                position: vertex_positions[index],
                uv: UVS[index],
            });
        }

        let vertex_ptr = renderer.start_copy_data_to_buffer(vertex_buffer_size, vertex_buffer_memory);

        unsafe {
            copy_nonoverlapping(vertices.as_ptr(), vertex_ptr.cast(), vertices.len());
        }

        renderer.end_copy_data_to_buffer(vertex_buffer_memory);

        let uniform_ptr = renderer.start_copy_data_to_buffer(uniform_buffer_size, uniform_buffer_memory);

        unsafe {
            let mut align = ash::util::Align::new(uniform_ptr, align_of::<f32>() as u64, RECT_DATA_SIZE);
            align.copy_from_slice(&[RectData {
                color: self.color,
                radius: self.radius,
                size: self.size,
            }]);
        }

        renderer.end_copy_data_to_buffer(uniform_buffer_memory);

        let descriptor_set_layout = renderer.create_descriptor_set_layout(RectData::get_descriptor_set_layout_bindings());
        let descriptor_set_layout_bindings = UvVertex::get_descriptor_set_layout_binding();
        let attribute_descriptions = UvVertex::get_attribute_descriptions();

        let (graphics_pipeline, graphics_pipeline_layout) = renderer.create_graphics_pipeline(
            vertex_shader,
            fragment_shader,
            viewport,
            scissor,
            render_pass,
            descriptor_set_layout,
            descriptor_set_layout_bindings,
            &attribute_descriptions,
        );

        let descriptor_pool = renderer.create_descriptor_pool(vec![*vk::DescriptorPoolSize::builder().ty(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(1)]);

        let descriptor_sets = renderer.create_descriptor_uniform_sets(descriptor_set_layout, descriptor_pool, uniform_buffer, RECT_DATA_SIZE);

        let command_buffer = renderer.create_command_buffer(command_pool);

        renderer.begin_render_pass(render_pass, frame_buffer, command_buffer, graphics_pipeline, viewport, scissor, width, height);

        unsafe {
            renderer.device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer], &[0]);
            renderer.device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT32);
            renderer
                .device
                .cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline_layout, 0, &descriptor_sets, &[]);
            renderer.device.cmd_draw_indexed(command_buffer, 6, 1, 0, 0, 1);
        }

        renderer.end_render_pass(command_buffer, graphics_queue);

        unsafe {
            renderer.device.destroy_descriptor_pool(descriptor_pool, None);

            renderer.device.destroy_pipeline(graphics_pipeline, None);
            renderer.device.destroy_pipeline_layout(graphics_pipeline_layout, None);

            renderer.device.destroy_descriptor_set_layout(descriptor_set_layout, None);
        }
    }
}

#[derive(Clone)]
pub struct Ellipse {
    pub position: Vector2<f32>,
    pub origin: Vector2<f32>,
    pub size: Vector2<f32>,
    pub color: Vector4<f32>,
    pub order: f32,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct EllipseData {
    color: Vector4<f32>,
}

pub const ELLIPSE_DATA_SIZE: u64 = 16;

impl EllipseData {
    fn get_descriptor_set_layout_bindings() -> Vec<vk::DescriptorSetLayoutBinding> {
        let layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build();
        vec![layout_binding]
    }
}

impl Ellipse {
    pub fn render(
        &self,
        renderer: &Renderer,
        graphics_queue: vk::Queue,
        render_pass: vk::RenderPass,
        command_pool: vk::CommandPool,
        frame_buffer: vk::Framebuffer,
        vertex_shader: ShaderModule,
        fragment_shader: ShaderModule,
        index_buffer: vk::Buffer,
        index_buffer_memory: vk::DeviceMemory,
        index_buffer_size: u64,
        vertex_buffer: vk::Buffer,
        vertex_buffer_memory: vk::DeviceMemory,
        vertex_buffer_size: u64,
        uniform_buffer: vk::Buffer,
        uniform_buffer_memory: vk::DeviceMemory,
        uniform_buffer_size: u64,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
        width: u32,
        height: u32,
        mode: RenderMode,
    ) {
        let index_ptr = renderer.start_copy_data_to_buffer(index_buffer_size, index_buffer_memory);

        unsafe {
            copy_nonoverlapping(vec![0, 1, 2, 2, 3, 0].as_ptr(), index_ptr.cast(), 6);
        }

        renderer.end_copy_data_to_buffer(index_buffer_memory);

        let normalize_scale = vec2(1920.0 / 2.0, 1080.0 / 2.0);

        let offsetted_x = self.position.x - self.origin.x * self.size.x;
        let offsetted_y = self.position.y - self.origin.y * self.size.y;

        let mut vertex_positions: Vec<Vector2<f32>> = vec![
            vec2(offsetted_x, offsetted_y),
            vec2(offsetted_x, offsetted_y + self.size.y),
            vec2(offsetted_x + self.size.x, offsetted_y + self.size.y),
            vec2(offsetted_x + self.size.x, offsetted_y),
        ];

        for vertex_position_index in 0..vertex_positions.len() {
            vertex_positions[vertex_position_index] = divide(vertex_positions[vertex_position_index], normalize_scale);
        }

        let mut vertices: Vec<UvVertex> = Vec::new();

        for index in 0..vertex_positions.len() {
            vertices.push(UvVertex {
                position: vertex_positions[index],
                uv: UVS[index],
            });
        }

        let vertex_ptr = renderer.start_copy_data_to_buffer(vertex_buffer_size, vertex_buffer_memory);

        unsafe {
            copy_nonoverlapping(vertices.as_ptr(), vertex_ptr.cast(), vertices.len());
        }

        renderer.end_copy_data_to_buffer(vertex_buffer_memory);

        let uniform_ptr = renderer.start_copy_data_to_buffer(uniform_buffer_size, uniform_buffer_memory);

        unsafe {
            let mut align = ash::util::Align::new(uniform_ptr, align_of::<f32>() as u64, ELLIPSE_DATA_SIZE);
            align.copy_from_slice(&[EllipseData { color: self.color }]);
        }

        renderer.end_copy_data_to_buffer(uniform_buffer_memory);

        let descriptor_set_layout = renderer.create_descriptor_set_layout(EllipseData::get_descriptor_set_layout_bindings());
        let descriptor_set_layout_bindings = UvVertex::get_descriptor_set_layout_binding();
        let attribute_descriptions = UvVertex::get_attribute_descriptions();

        let (graphics_pipeline, graphics_pipeline_layout) = renderer.create_graphics_pipeline(
            vertex_shader,
            fragment_shader,
            viewport,
            scissor,
            render_pass,
            descriptor_set_layout,
            descriptor_set_layout_bindings,
            &attribute_descriptions,
        );

        let descriptor_pool = renderer.create_descriptor_pool(vec![*vk::DescriptorPoolSize::builder().ty(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(1)]);

        let descriptor_sets = renderer.create_descriptor_uniform_sets(descriptor_set_layout, descriptor_pool, uniform_buffer, ELLIPSE_DATA_SIZE);

        let command_buffer = renderer.create_command_buffer(command_pool);

        renderer.begin_render_pass(render_pass, frame_buffer, command_buffer, graphics_pipeline, viewport, scissor, width, height);

        unsafe {
            renderer.device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer], &[0]);
            renderer.device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT32);
            renderer
                .device
                .cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline_layout, 0, &descriptor_sets, &[]);
            renderer.device.cmd_draw_indexed(command_buffer, 6, 1, 0, 0, 1);
        }

        renderer.end_render_pass(command_buffer, graphics_queue);

        unsafe {
            renderer.device.destroy_descriptor_pool(descriptor_pool, None);

            renderer.device.destroy_pipeline(graphics_pipeline, None);
            renderer.device.destroy_pipeline_layout(graphics_pipeline_layout, None);

            renderer.device.destroy_descriptor_set_layout(descriptor_set_layout, None);
        }
    }
}

#[derive(Clone)]
pub struct Clip {
    pub clip: String,
    pub frame: u32,
    pub position: Vector2<f32>,
    pub origin: Vector2<f32>,
    pub size: Vector2<f32>,
    pub rotation: f32,
    pub color: Vector4<f32>,
    pub order: f32,
}

#[derive(Clone, Copy)]
pub struct ClipData {
    color: Vector4<f32>,
    size: Vector2<f32>,
}

pub const CLIP_DATA_SIZE: u64 = 16 + 8;

impl ClipData {
    fn get_descriptor_set_layout_bindings() -> Vec<vk::DescriptorSetLayoutBinding> {
        let layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build();

        let sampler_binding = *vk::DescriptorSetLayoutBinding::builder()
            .binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);

        vec![layout_binding, sampler_binding]
    }
}

impl Clip {
    pub fn render(
        &self,
        renderer: &Renderer,
        graphics_queue: vk::Queue,
        render_pass: vk::RenderPass,
        command_pool: vk::CommandPool,
        frame_buffer: vk::Framebuffer,
        vertex_shader: ShaderModule,
        fragment_shader: ShaderModule,
        index_buffer: vk::Buffer,
        index_buffer_memory: vk::DeviceMemory,
        index_buffer_size: u64,
        vertex_buffer: vk::Buffer,
        vertex_buffer_memory: vk::DeviceMemory,
        vertex_buffer_size: u64,
        uniform_buffer: vk::Buffer,
        uniform_buffer_memory: vk::DeviceMemory,
        uniform_buffer_size: u64,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
        width: u32,
        height: u32,
        clip_loader: &mut ClipLoader,
        mode: RenderMode,
    ) {
        let clip = clip_loader.get(&self.clip, renderer).unwrap();
        let clip = &mut *clip.borrow_mut();

        let clip_image_view = match clip {
            Clips::ScriptClip(ref mut clip) => {
                clip.set_frame(self.frame);

                clip.render(renderer, clip_loader, self.size.x as u32, self.size.y as u32, RenderMode::Sample).image_view
            }
            Clips::ImageClip(ref mut clip) => clip.render(renderer, clip_loader).image_view,
        };

        let index_ptr = renderer.start_copy_data_to_buffer(index_buffer_size, index_buffer_memory);

        unsafe {
            copy_nonoverlapping(vec![0, 1, 2, 2, 3, 0].as_ptr(), index_ptr.cast(), 6);
        }

        renderer.end_copy_data_to_buffer(index_buffer_memory);

        let normalize_scale = vec2(1920.0 / 2.0, 1080.0 / 2.0);

        let offsetted_x = self.position.x - self.origin.x * self.size.x;
        let offsetted_y = self.position.y - self.origin.y * self.size.y;

        let mut vertex_positions: Vec<Vector2<f32>> = vec![
            vec2(offsetted_x, offsetted_y),
            vec2(offsetted_x, offsetted_y + self.size.y),
            vec2(offsetted_x + self.size.x, offsetted_y + self.size.y),
            vec2(offsetted_x + self.size.x, offsetted_y),
        ];

        for vertex_position_index in 0..vertex_positions.len() {
            vertex_positions[vertex_position_index] = flip_vertically(divide(rotate(vertex_positions[vertex_position_index], self.position, self.rotation), normalize_scale));
        }

        let mut vertices: Vec<UvVertex> = Vec::new();

        for index in 0..vertex_positions.len() {
            vertices.push(UvVertex {
                position: vertex_positions[index],
                uv: UVS[index],
            });
        }

        let vertex_ptr = renderer.start_copy_data_to_buffer(vertex_buffer_size, vertex_buffer_memory);

        unsafe {
            copy_nonoverlapping(vertices.as_ptr(), vertex_ptr.cast(), vertices.len());
        }

        renderer.end_copy_data_to_buffer(vertex_buffer_memory);

        let uniform_ptr = renderer.start_copy_data_to_buffer(uniform_buffer_size, uniform_buffer_memory);

        unsafe {
            let mut align = ash::util::Align::new(uniform_ptr, align_of::<f32>() as u64, CLIP_DATA_SIZE);
            align.copy_from_slice(&[ClipData { color: self.color, size: self.size }]);
        }

        renderer.end_copy_data_to_buffer(uniform_buffer_memory);

        let descriptor_set_layout = renderer.create_descriptor_set_layout(ClipData::get_descriptor_set_layout_bindings());
        let descriptor_set_layout_bindings = UvVertex::get_descriptor_set_layout_binding();
        let attribute_descriptions = UvVertex::get_attribute_descriptions();

        let (graphics_pipeline, graphics_pipeline_layout) = renderer.create_graphics_pipeline(
            vertex_shader,
            fragment_shader,
            viewport,
            scissor,
            render_pass,
            descriptor_set_layout,
            descriptor_set_layout_bindings,
            &attribute_descriptions,
        );

        let descriptor_pool = renderer.create_descriptor_pool(vec![*vk::DescriptorPoolSize::builder().ty(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(1)]);

        let sampler = renderer.create_sampler();

        let descriptor_sets = renderer.create_descriptor_uniform_sampler_sets(descriptor_set_layout, descriptor_pool, uniform_buffer, clip_image_view, sampler, CLIP_DATA_SIZE);

        let command_buffer = renderer.create_command_buffer(command_pool);

        renderer.begin_render_pass(render_pass, frame_buffer, command_buffer, graphics_pipeline, viewport, scissor, width, height);

        unsafe {
            renderer.device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer], &[0]);
            renderer.device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT32);
            renderer
                .device
                .cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline_layout, 0, &descriptor_sets, &[]);
            renderer.device.cmd_draw_indexed(command_buffer, 6, 1, 0, 0, 1);
        }

        renderer.end_render_pass(command_buffer, graphics_queue);

        unsafe {
            renderer.device.destroy_sampler(sampler, None);

            renderer.device.destroy_descriptor_pool(descriptor_pool, None);

            renderer.device.destroy_pipeline(graphics_pipeline, None);
            renderer.device.destroy_pipeline_layout(graphics_pipeline_layout, None);

            renderer.device.destroy_descriptor_set_layout(descriptor_set_layout, None);
        }
    }
}

#[derive(Clone)]
pub struct FontAtlas {
    pub path: String,
    pub rows: u32,
    pub columns: u32,
    pub dropdown: f32,
    pub spacing: f32,
    pub characters: String,
    pub width_overrides: HashMap<char, f32>,
}

#[derive(Clone)]
pub struct Text {
    pub text: String,
    pub font: FontAtlas,
    pub position: Vector2<f32>,
    pub origin: Vector2<f32>,
    pub size: f32,
    pub rotation: f32,
    pub color: Vector4<f32>,
    pub order: f32,
}

#[derive(Clone, Copy)]
pub struct TextData {
    color: Vector4<f32>,
}

pub const TEXT_DATA_SIZE: u64 = 16;

impl TextData {
    fn get_descriptor_set_layout_bindings() -> Vec<vk::DescriptorSetLayoutBinding> {
        let layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build();

        let sampler_binding = *vk::DescriptorSetLayoutBinding::builder()
            .binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);

        vec![layout_binding, sampler_binding]
    }
}

impl Text {
    pub fn render(
        &self,
        renderer: &Renderer,
        graphics_queue: vk::Queue,
        render_pass: vk::RenderPass,
        command_pool: vk::CommandPool,
        frame_buffer: vk::Framebuffer,
        vertex_shader: ShaderModule,
        fragment_shader: ShaderModule,
        index_buffer: vk::Buffer,
        index_buffer_memory: vk::DeviceMemory,
        index_buffer_size: u64,
        vertex_buffer: vk::Buffer,
        vertex_buffer_memory: vk::DeviceMemory,
        vertex_buffer_size: u64,
        uniform_buffer: vk::Buffer,
        uniform_buffer_memory: vk::DeviceMemory,
        uniform_buffer_size: u64,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
        width: u32,
        height: u32,
        clip_loader: &mut ClipLoader,
        mode: RenderMode,
    ) {
        let clip = clip_loader.get(&self.font.path, renderer).unwrap();
        let clip = &mut *clip.borrow_mut();

        let atlas = match clip {
            Clips::ImageClip(ref mut clip) => clip.render(renderer, clip_loader),
            _ => {
                panic!("Font atlas can not render from this clip type!")
            }
        };

        let mut indices: Vec<u32> = Vec::new();

        for index in 0..self.text.len() as u32 {
            indices.extend_from_slice(&[index * 4, index * 4 + 1, index * 4 + 2, index * 4 + 2, index * 4 + 3, index * 4])
        }

        let index_ptr = renderer.start_copy_data_to_buffer(index_buffer_size, index_buffer_memory);

        unsafe {
            copy_nonoverlapping(indices.as_ptr(), index_ptr.cast(), indices.len());
        }

        renderer.end_copy_data_to_buffer(index_buffer_memory);

        let cell_size = atlas.width as f32 / self.font.columns as f32;
        let character_width = cell_size * self.font.spacing;
        let scale_factor = self.size / character_width;

        let cell_drop_vertex = cell_size * self.font.dropdown * scale_factor;
        let cell_drop_uv = self.font.dropdown * 1.0 / self.font.rows as f32;

        let normalize_scale = vec2(1920.0 / 2.0, 1080.0 / 2.0);

        let mut vertex_positions: Vec<Vector2<f32>> = Vec::new();

        let mut calculated_width: f32 = 0.0;

        let chars = self.text.chars().collect::<Vec<char>>();

        for index in 0..self.text.len() as u32 {
            if self.font.width_overrides.contains_key(&chars[index as usize]) {
                calculated_width += cell_size * self.font.width_overrides.get(&chars[index as usize]).unwrap() * scale_factor;
            } else {
                calculated_width += character_width * scale_factor;
            }
        }

        let mut character_x: f32 = 0.0;

        let offsetted_x = self.position.x - calculated_width * self.origin.x;
        let offsetted_y = self.position.y - (character_width * scale_factor) * self.origin.y;

        for index in 0..self.text.len() as u32 {
            vertex_positions.extend_from_slice(&[
                vec2(offsetted_x + character_x as f32, offsetted_y - cell_drop_vertex),
                vec2(offsetted_x + character_x as f32, offsetted_y + cell_size * scale_factor - cell_drop_vertex),
                vec2(offsetted_x + character_x as f32 + cell_size * scale_factor, offsetted_y + cell_size * scale_factor - cell_drop_vertex),
                vec2(offsetted_x + character_x as f32 + cell_size * scale_factor, offsetted_y - cell_drop_vertex),
            ]);

            if self.font.width_overrides.contains_key(&chars[index as usize]) {
                character_x += cell_size * self.font.width_overrides.get(&chars[index as usize]).unwrap() * scale_factor;
            } else {
                character_x += character_width * scale_factor;
            }
        }

        for vertex_position_index in 0..vertex_positions.len() {
            vertex_positions[vertex_position_index] = flip_vertically(divide(rotate(vertex_positions[vertex_position_index], self.position, self.rotation), normalize_scale));
        }

        let mut vertices: Vec<UvVertex> = Vec::new();
        for index in 0..vertex_positions.len() {
            let face_index = index / 4;

            let character_index = self.font.characters.find(chars[face_index]).unwrap() as u32;

            let start_u = (character_index % self.font.columns) as f32 / self.font.columns as f32;
            let start_v = (character_index / self.font.columns) as f32 / self.font.rows as f32;

            let u_size = 1.0 / self.font.columns as f32;
            let v_size = 1.0 / self.font.rows as f32;

            vertices.push(UvVertex {
                position: vertex_positions[index],
                uv: vec2(start_u + u_size * UVS[index % 4].x, start_v + v_size * UVS[index % 4].y + cell_drop_uv),
            });
        }

        let vertex_ptr = renderer.start_copy_data_to_buffer(vertex_buffer_size, vertex_buffer_memory);

        unsafe {
            copy_nonoverlapping(vertices.as_ptr(), vertex_ptr.cast(), vertices.len());
        }

        renderer.end_copy_data_to_buffer(vertex_buffer_memory);

        let uniform_ptr = renderer.start_copy_data_to_buffer(uniform_buffer_size, uniform_buffer_memory);

        unsafe {
            let mut align = ash::util::Align::new(uniform_ptr, align_of::<f32>() as u64, TEXT_DATA_SIZE);
            align.copy_from_slice(&[TextData { color: self.color }]);
        }

        renderer.end_copy_data_to_buffer(uniform_buffer_memory);

        let descriptor_set_layout = renderer.create_descriptor_set_layout(TextData::get_descriptor_set_layout_bindings());
        let descriptor_set_layout_bindings = UvVertex::get_descriptor_set_layout_binding();
        let attribute_descriptions = UvVertex::get_attribute_descriptions();

        let (graphics_pipeline, graphics_pipeline_layout) = renderer.create_graphics_pipeline(
            vertex_shader,
            fragment_shader,
            viewport,
            scissor,
            render_pass,
            descriptor_set_layout,
            descriptor_set_layout_bindings,
            &attribute_descriptions,
        );

        let descriptor_pool = renderer.create_descriptor_pool(vec![*vk::DescriptorPoolSize::builder().ty(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(1)]);

        let sampler = renderer.create_sampler();

        let descriptor_sets = renderer.create_descriptor_uniform_sampler_sets(descriptor_set_layout, descriptor_pool, uniform_buffer, atlas.image_view, sampler, TEXT_DATA_SIZE);

        let command_buffer = renderer.create_command_buffer(command_pool);

        renderer.begin_render_pass(render_pass, frame_buffer, command_buffer, graphics_pipeline, viewport, scissor, width, height);

        unsafe {
            renderer.device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer], &[0]);
            renderer.device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT32);
            renderer
                .device
                .cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline_layout, 0, &descriptor_sets, &[]);
            renderer.device.cmd_draw_indexed(command_buffer, 6 * self.text.len() as u32, 1, 0, 0, 1);
        }

        renderer.end_render_pass(command_buffer, graphics_queue);

        unsafe {
            renderer.device.destroy_sampler(sampler, None);

            renderer.device.destroy_descriptor_pool(descriptor_pool, None);

            renderer.device.destroy_pipeline(graphics_pipeline, None);
            renderer.device.destroy_pipeline_layout(graphics_pipeline_layout, None);

            renderer.device.destroy_descriptor_set_layout(descriptor_set_layout, None);
        }
    }

    pub fn calculate_size(&self) -> Vector2<f32> {
        vec2(200.0, 200.0)
    }
}
