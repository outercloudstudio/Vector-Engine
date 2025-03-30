use ash::vk::{Framebuffer, RenderPass, ShaderModule};
use ash::{vk, Device};
use cgmath::{vec2, Vector2, Vector4};
use log::info;
use std::mem::align_of;
use std::ptr::copy_nonoverlapping;

use super::renderer::RenderTarget;
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
}

impl Elements {
    pub fn get_order(&self) -> f32 {
        match &self {
            Elements::Rect(rect) => rect.order,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UvVertex {
    pub position: Vector2<f32>,
    pub uv: Vector2<f32>,
}

impl UvVertex {
    pub fn get_descriptor_set_layout_binding() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(size_of::<UvVertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        let position = vk::VertexInputAttributeDescription::default().binding(0).location(0).format(vk::Format::R32G32_SFLOAT).offset(0);

        let uv = vk::VertexInputAttributeDescription::default().binding(0).location(1).format(vk::Format::R32G32_SFLOAT).offset(8);

        [position, uv]
    }
}

pub struct ElementRenderContext {
    device: Device,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
    rect_render_context: RectRenderContext,
}

impl ElementRenderContext {
    pub fn new(renderer: &Renderer) -> ElementRenderContext {
        let graphics_queue = renderer.create_graphics_queue();
        let command_pool = renderer.create_command_pool();

        let rect_render_context = RectRenderContext::new(renderer);

        ElementRenderContext {
            graphics_queue,
            command_pool,
            device: renderer.device.clone(),

            rect_render_context,
        }
    }
}

impl Drop for ElementRenderContext {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_command_pool(self.command_pool, None);
        }
    }
}

pub struct PatchRenderContext {
    device: Device,

    width: u32,
    height: u32,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct RectRenderContext {
    device: Device,

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
}

impl RectRenderContext {
    pub fn new(renderer: &Renderer) -> RectRenderContext {
        let device = renderer.device.clone();

        let vertex_shader = renderer.create_shader(include_bytes!("./shaders/compiled/rect.vert.spv").to_vec());
        let fragment_shader = renderer.create_shader(include_bytes!("./shaders/compiled/rect.frag.spv").to_vec());

        let (index_buffer, index_buffer_memory, index_buffer_size) = renderer.create_buffer(
            4 * 6,
            vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        let (vertex_buffer, vertex_buffer_memory, vertex_buffer_size) = renderer.create_buffer(
            size_of::<UvVertex>() as u64 * 4,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        let (uniform_buffer, uniform_buffer_memory, uniform_buffer_size) = renderer.create_buffer(
            size_of::<RectData>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        let index_ptr = renderer.start_copy_data_to_buffer(index_buffer_size, index_buffer_memory);

        unsafe {
            copy_nonoverlapping(vec![0, 1, 2, 2, 3, 0].as_ptr(), index_ptr.cast(), 6);
        }

        renderer.end_copy_data_to_buffer(index_buffer_memory);

        let vertex_positions: Vec<Vector2<f32>> = vec![vec2(0_f32, 0_f32), vec2(0_f32, 1_f32), vec2(1_f32, 1_f32), vec2(1_f32, 0_f32)];

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

        return RectRenderContext {
            device,
            vertex_shader,
            fragment_shader,
            index_buffer,
            index_buffer_memory,
            index_buffer_size,
            vertex_buffer,
            vertex_buffer_memory,
            vertex_buffer_size,
            uniform_buffer,
            uniform_buffer_memory,
            uniform_buffer_size,
        };
    }
}

impl Drop for RectRenderContext {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_shader_module(self.vertex_shader, None);
            self.device.destroy_shader_module(self.fragment_shader, None);

            self.device.destroy_buffer(self.index_buffer, None);
            self.device.free_memory(self.index_buffer_memory, None);

            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_buffer_memory, None);

            self.device.destroy_buffer(self.uniform_buffer, None);
            self.device.free_memory(self.uniform_buffer_memory, None);
        }
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
    pub position: Vector2<f32>,
    pub origin: Vector2<f32>,
    pub size: Vector2<f32>,
    pub radius: f32,
    pub rotation: f32,
}

impl RectData {
    pub fn get_descriptor_set_layout_bindings() -> Vec<vk::DescriptorSetLayoutBinding<'static>> {
        let layout_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS);
        vec![layout_binding]
    }
}

// TODO: Look at push constants
impl Rect {
    pub fn render(&self, renderer: &Renderer, element_render_context: &ElementRenderContext, render_target: &RenderTarget) {
        let uniform_ptr = renderer.start_copy_data_to_buffer(
            element_render_context.rect_render_context.uniform_buffer_size,
            element_render_context.rect_render_context.uniform_buffer_memory,
        );

        unsafe {
            let mut align = ash::util::Align::new(uniform_ptr, align_of::<f32>() as u64, size_of::<RectData>() as u64);
            align.copy_from_slice(&[RectData {
                color: self.color,
                position: self.position,
                origin: self.origin,
                radius: self.radius,
                size: self.size,
                rotation: self.rotation,
            }]);
        }

        renderer.end_copy_data_to_buffer(element_render_context.rect_render_context.uniform_buffer_memory);

        let descriptor_set_layout = renderer.create_descriptor_set_layout(RectData::get_descriptor_set_layout_bindings());
        let descriptor_set_layout_bindings = UvVertex::get_descriptor_set_layout_binding();
        let attribute_descriptions = UvVertex::get_attribute_descriptions();

        let (graphics_pipeline, graphics_pipeline_layout) = renderer.create_graphics_pipeline(
            element_render_context.rect_render_context.vertex_shader,
            element_render_context.rect_render_context.fragment_shader,
            render_target.viewport,
            render_target.scissor,
            render_target.render_pass,
            descriptor_set_layout,
            descriptor_set_layout_bindings,
            &attribute_descriptions,
        );

        let descriptor_pool = renderer.create_descriptor_pool(vec![vk::DescriptorPoolSize::default().ty(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(1)]);

        let descriptor_sets = renderer.create_descriptor_uniform_sets(
            descriptor_set_layout,
            descriptor_pool,
            element_render_context.rect_render_context.uniform_buffer,
            size_of::<RectData>() as u64,
        );

        let command_buffer = renderer.create_command_buffer(element_render_context.command_pool);

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

        unsafe {
            renderer
                .device
                .cmd_bind_vertex_buffers(command_buffer, 0, &[element_render_context.rect_render_context.vertex_buffer], &[0]);
            renderer
                .device
                .cmd_bind_index_buffer(command_buffer, element_render_context.rect_render_context.index_buffer, 0, vk::IndexType::UINT32);
            renderer
                .device
                .cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline_layout, 0, &descriptor_sets, &[]);
            renderer.device.cmd_draw_indexed(command_buffer, 6, 1, 0, 0, 1);
        }

        renderer.end_render_pass(command_buffer, element_render_context.graphics_queue);
        renderer.execute_render_pass(command_buffer, element_render_context.graphics_queue);

        unsafe {
            renderer.device.destroy_descriptor_pool(descriptor_pool, None);

            renderer.device.destroy_pipeline(graphics_pipeline, None);
            renderer.device.destroy_pipeline_layout(graphics_pipeline_layout, None);

            renderer.device.destroy_descriptor_set_layout(descriptor_set_layout, None);
        }
    }
}
