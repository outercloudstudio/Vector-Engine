use ash::util::read_spv;
use ash::vk::ShaderModule;
use ash::{vk, Device, Instance};
use cgmath::{vec2, Vector2, Vector4};
use std::io::Cursor;
use std::mem::align_of;
use std::ptr::copy_nonoverlapping;

use crate::clips::{ClipLoader, Clips};
use crate::renderer::{utils::*, Renderer};

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
}

impl Elements {
    pub fn get_order(&self) -> f32 {
        match &self {
            Elements::Rect(rect) => rect.order,
            Elements::Ellipse(ellipse) => ellipse.order,
            Elements::Clip(clip) => clip.order,
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

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct RectVertex {
    position: Vector2<f32>,
    uv: Vector2<f32>,
}
const RECT_VERTEX_SIZE: u64 = 8 + 8;

impl RectVertex {
    pub fn get_descriptor_set_layout_binding() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(RECT_VERTEX_SIZE as u32)
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

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct RectData {
    color: Vector4<f32>,
    size: Vector2<f32>,
    radius: f32,
}
const RECT_DATA_SIZE: u64 = 16 + 4 + 8;

impl RectData {
    fn get_descriptor_set_layout_bindings() -> [vk::DescriptorSetLayoutBinding; 1] {
        let layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build();
        [layout_binding]
    }
}

impl Rect {
    fn create_vertex_shader(device: &Device) -> ShaderModule {
        unsafe {
            let mut vertex_spv_file = Cursor::new(&include_bytes!("./shaders/compiled/rect.vert.spv"));

            let vertex_code = read_spv(&mut vertex_spv_file).expect("Failed to read vertex shader spv file");
            let vertex_shader_info = vk::ShaderModuleCreateInfo::builder().code(&vertex_code);

            device.create_shader_module(&vertex_shader_info, None).expect("Vertex shader module error")
        }
    }

    fn create_fragment_shader(device: &Device) -> ShaderModule {
        unsafe {
            let mut frag_spv_file = Cursor::new(&include_bytes!("./shaders/compiled/rect.frag.spv"));

            let frag_code = read_spv(&mut frag_spv_file).expect("Failed to read fragment shader spv file");
            let frag_shader_info = vk::ShaderModuleCreateInfo::builder().code(&frag_code);

            device.create_shader_module(&frag_shader_info, None).expect("Fragment shader module error")
        }
    }

    fn create_vertex_buffer(vertex_positions: &Vec<Vector2<f32>>, instance: &Instance, device: &Device, physical_device: vk::PhysicalDevice) -> (vk::Buffer, vk::DeviceMemory) {
        unsafe {
            let mut vertices: Vec<RectVertex> = Vec::new();

            for index in 0..(vertex_positions.len()) {
                vertices.push(RectVertex {
                    position: vertex_positions[index],
                    uv: UVS[index],
                });
            }

            let vertex_buffer_size = RECT_VERTEX_SIZE * vertices.len() as u64;

            let vertex_buffer_info = *vk::BufferCreateInfo::builder()
                .size(vertex_buffer_size)
                .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let vertex_buffer = device.create_buffer(&vertex_buffer_info, None).unwrap();

            let vertex_buffer_memory_requirements = device.get_buffer_memory_requirements(vertex_buffer);
            let vertex_buffer_memory_index = get_memory_type_index(
                &instance,
                physical_device,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                vertex_buffer_memory_requirements,
            );

            let vertex_buffer_allocate_info = *vk::MemoryAllocateInfo::builder()
                .allocation_size(vertex_buffer_memory_requirements.size)
                .memory_type_index(vertex_buffer_memory_index);

            let vertex_buffer_memory = device.allocate_memory(&vertex_buffer_allocate_info, None).unwrap();

            let vert_ptr = device.map_memory(vertex_buffer_memory, 0, vertex_buffer_memory_requirements.size, vk::MemoryMapFlags::empty()).unwrap();

            copy_nonoverlapping(vertices.as_ptr(), vert_ptr.cast(), vertices.len());

            device.unmap_memory(vertex_buffer_memory);
            device.bind_buffer_memory(vertex_buffer, vertex_buffer_memory, 0).unwrap();

            return (vertex_buffer, vertex_buffer_memory);
        }
    }

    fn create_descriptor_set_layout(device: &Device) -> vk::DescriptorSetLayout {
        let bindings = RectData::get_descriptor_set_layout_bindings();
        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings).build();

        unsafe { device.create_descriptor_set_layout(&layout_info, None).unwrap() }
    }

    fn create_descriptor_pool(device: &Device) -> vk::DescriptorPool {
        unsafe {
            let uniform_buffer_object_size = *vk::DescriptorPoolSize::builder().ty(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(1);

            let pool_sizes = [uniform_buffer_object_size];
            let info = vk::DescriptorPoolCreateInfo::builder().pool_sizes(&pool_sizes).max_sets(1);

            device.create_descriptor_pool(&info, None).unwrap()
        }
    }

    fn create_descriptor_sets(device: &Device, descriptor_set_layout: vk::DescriptorSetLayout, descriptor_pool: vk::DescriptorPool, uniform_buffer: vk::Buffer) -> Vec<vk::DescriptorSet> {
        unsafe {
            let layouts = vec![descriptor_set_layout; 1];
            let info = vk::DescriptorSetAllocateInfo::builder().descriptor_pool(descriptor_pool).set_layouts(&layouts);

            let descriptor_sets = device.allocate_descriptor_sets(&info).unwrap();

            // Range is the size of the RectDataStruct
            let info = *vk::DescriptorBufferInfo::builder().buffer(uniform_buffer).offset(0).range(RECT_DATA_SIZE);

            let buffer_info = &[info];
            let ubo_write = *vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[0])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(buffer_info);

            device.update_descriptor_sets(&[ubo_write], &[] as &[vk::CopyDescriptorSet]);

            return descriptor_sets;
        }
    }

    fn create_uniform_buffer(data: RectData, instance: &Instance, device: &Device, physical_device: vk::PhysicalDevice) -> (vk::Buffer, vk::DeviceMemory) {
        unsafe {
            let uniform_buffer_object = data;

            let uniform_buffer_size = RECT_DATA_SIZE;

            let unfiorm_buffer_info = *vk::BufferCreateInfo::builder()
                .size(uniform_buffer_size)
                .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let uniform_buffer = device.create_buffer(&unfiorm_buffer_info, None).unwrap();

            let uniform_buffer_memory_requirements = device.get_buffer_memory_requirements(uniform_buffer);
            let uniform_buffer_memory_index = get_memory_type_index(
                &instance,
                physical_device,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                uniform_buffer_memory_requirements,
            );

            let uniform_buffer_allocate_info = *vk::MemoryAllocateInfo::builder()
                .allocation_size(uniform_buffer_memory_requirements.size)
                .memory_type_index(uniform_buffer_memory_index);

            let uniform_buffer_memory = device.allocate_memory(&uniform_buffer_allocate_info, None).unwrap();

            let uniform_ptr = device
                .map_memory(uniform_buffer_memory, 0, uniform_buffer_memory_requirements.size, vk::MemoryMapFlags::empty())
                .unwrap();

            let mut align = ash::util::Align::new(uniform_ptr, align_of::<f32>() as u64, RECT_DATA_SIZE);
            align.copy_from_slice(&[uniform_buffer_object]);

            device.unmap_memory(uniform_buffer_memory);
            device.bind_buffer_memory(uniform_buffer, uniform_buffer_memory, 0).unwrap();

            return (uniform_buffer, uniform_buffer_memory);
        }
    }

    pub fn render(
        &self,
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
        target_image_view: vk::ImageView,
        command_pool: vk::CommandPool,
        graphics_queue: vk::Queue,
        first_element: bool,
        last_element: bool,
        width: u32,
        height: u32,
    ) {
        let command_buffer = create_command_buffer(device, command_pool);

        let render_pass = create_render_pass(device, first_element, last_element);

        let frame_buffer = create_framebuffer(device, target_image_view, render_pass, width, height);

        let vertex_shader = Rect::create_vertex_shader(device);
        let fragment_shader = Rect::create_fragment_shader(device);

        let viewport = create_viewport(width, height);
        let scissor = create_scissor(width, height);

        let descriptor_set_layouts = Rect::create_descriptor_set_layout(device);

        let (graphics_pipeline, graphics_pipeline_layout) = create_graphics_pipeline(
            device,
            vertex_shader,
            fragment_shader,
            viewport,
            scissor,
            render_pass,
            descriptor_set_layouts,
            RectVertex::get_descriptor_set_layout_binding(),
            &RectVertex::get_attribute_descriptions(),
        );

        let (index_buffer, index_buffer_memory) = create_index_buffer(&vec![0, 1, 2, 2, 3, 0], instance, device, physical_device);

        let normalize_scale = vec2(1920.0 / 2.0, 1080.0 / 2.0);

        let offsetted_x = self.position.x - self.origin.x * self.size.x;
        let offsetted_y = self.position.y - self.origin.y * self.size.y;

        let mut vertices = vec![
            vec2(offsetted_x, offsetted_y),
            vec2(offsetted_x, offsetted_y + self.size.y),
            vec2(offsetted_x + self.size.x, offsetted_y + self.size.y),
            vec2(offsetted_x + self.size.x, offsetted_y),
        ];

        for vertex_index in 0..vertices.len() {
            vertices[vertex_index] = flip_vertically(divide(rotate(vertices[vertex_index], self.position, self.rotation), normalize_scale))
        }

        let (vertex_buffer, vertex_buffer_memory) = Rect::create_vertex_buffer(&vertices, instance, device, physical_device);

        let (uniform_buffer, uniform_buffer_memory) = Rect::create_uniform_buffer(
            RectData {
                color: self.color,
                radius: self.radius,
                size: self.size,
            },
            instance,
            device,
            physical_device,
        );

        let descriptor_pools = Rect::create_descriptor_pool(device);
        let descriptor_sets = Rect::create_descriptor_sets(device, descriptor_set_layouts, descriptor_pools, uniform_buffer);

        begin_render_pass(device, render_pass, frame_buffer, command_buffer, graphics_pipeline, viewport, scissor, width, height);

        unsafe {
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer], &[0]);
            device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT32);
            device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline_layout, 0, &descriptor_sets, &[]);
            device.cmd_draw_indexed(command_buffer, 6, 1, 0, 0, 1);
        }

        end_render_pass(device, command_buffer, graphics_queue);

        unsafe {
            device.destroy_shader_module(vertex_shader, None);
            device.destroy_shader_module(fragment_shader, None);

            device.destroy_descriptor_pool(descriptor_pools, None);

            device.destroy_framebuffer(frame_buffer, None);

            device.destroy_pipeline(graphics_pipeline, None);
            device.destroy_pipeline_layout(graphics_pipeline_layout, None);

            device.destroy_render_pass(render_pass, None);

            device.destroy_descriptor_set_layout(descriptor_set_layouts, None);

            device.destroy_buffer(index_buffer, None);
            device.free_memory(index_buffer_memory, None);

            device.destroy_buffer(vertex_buffer, None);
            device.free_memory(vertex_buffer_memory, None);

            device.destroy_buffer(uniform_buffer, None);
            device.free_memory(uniform_buffer_memory, None);
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

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct EllipseVertex {
    position: Vector2<f32>,
    uv: Vector2<f32>,
}
const ELLIPSE_VERTEX_SIZE: u64 = 8 + 8;

impl EllipseVertex {
    pub fn get_descriptor_set_layout_binding() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(ELLIPSE_VERTEX_SIZE as u32)
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

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct EllipseData {
    color: Vector4<f32>,
}
const ELLIPSE_DATA_SIZE: u64 = 16;

impl EllipseData {
    fn get_descriptor_set_layout_bindings() -> [vk::DescriptorSetLayoutBinding; 1] {
        let layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build();
        [layout_binding]
    }
}

impl Ellipse {
    fn create_vertex_shader(device: &Device) -> ShaderModule {
        unsafe {
            let mut vertex_spv_file = Cursor::new(&include_bytes!("./shaders/compiled/ellipse.vert.spv"));

            let vertex_code = read_spv(&mut vertex_spv_file).expect("Failed to read vertex shader spv file");
            let vertex_shader_info = vk::ShaderModuleCreateInfo::builder().code(&vertex_code);

            device.create_shader_module(&vertex_shader_info, None).expect("Vertex shader module error")
        }
    }

    fn create_fragment_shader(device: &Device) -> ShaderModule {
        unsafe {
            let mut frag_spv_file = Cursor::new(&include_bytes!("./shaders/compiled/ellipse.frag.spv"));

            let frag_code = read_spv(&mut frag_spv_file).expect("Failed to read fragment shader spv file");
            let frag_shader_info = vk::ShaderModuleCreateInfo::builder().code(&frag_code);

            device.create_shader_module(&frag_shader_info, None).expect("Fragment shader module error")
        }
    }

    fn create_vertex_buffer(vertex_positions: &Vec<Vector2<f32>>, instance: &Instance, device: &Device, physical_device: vk::PhysicalDevice) -> (vk::Buffer, vk::DeviceMemory) {
        unsafe {
            let mut vertices: Vec<EllipseVertex> = Vec::new();

            for index in 0..(vertex_positions.len()) {
                vertices.push(EllipseVertex {
                    position: vertex_positions[index],
                    uv: UVS[index],
                });
            }

            let vertex_buffer_size = ELLIPSE_VERTEX_SIZE * vertices.len() as u64;

            let vertex_buffer_info = *vk::BufferCreateInfo::builder()
                .size(vertex_buffer_size)
                .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let vertex_buffer = device.create_buffer(&vertex_buffer_info, None).unwrap();

            let vertex_buffer_memory_requirements = device.get_buffer_memory_requirements(vertex_buffer);
            let vertex_buffer_memory_index = get_memory_type_index(
                &instance,
                physical_device,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                vertex_buffer_memory_requirements,
            );

            let vertex_buffer_allocate_info = *vk::MemoryAllocateInfo::builder()
                .allocation_size(vertex_buffer_memory_requirements.size)
                .memory_type_index(vertex_buffer_memory_index);

            let vertex_buffer_memory = device.allocate_memory(&vertex_buffer_allocate_info, None).unwrap();

            let vert_ptr = device.map_memory(vertex_buffer_memory, 0, vertex_buffer_memory_requirements.size, vk::MemoryMapFlags::empty()).unwrap();

            copy_nonoverlapping(vertices.as_ptr(), vert_ptr.cast(), vertices.len());

            device.unmap_memory(vertex_buffer_memory);
            device.bind_buffer_memory(vertex_buffer, vertex_buffer_memory, 0).unwrap();

            return (vertex_buffer, vertex_buffer_memory);
        }
    }

    fn create_descriptor_set_layout(device: &Device) -> vk::DescriptorSetLayout {
        let bindings = EllipseData::get_descriptor_set_layout_bindings();
        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings).build();

        unsafe { device.create_descriptor_set_layout(&layout_info, None).unwrap() }
    }

    fn create_descriptor_pool(device: &Device) -> vk::DescriptorPool {
        unsafe {
            let uniform_buffer_object_size = *vk::DescriptorPoolSize::builder().ty(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(1);

            let pool_sizes = [uniform_buffer_object_size];
            let info = vk::DescriptorPoolCreateInfo::builder().pool_sizes(&pool_sizes).max_sets(1);

            device.create_descriptor_pool(&info, None).unwrap()
        }
    }

    fn create_descriptor_sets(device: &Device, descriptor_set_layout: vk::DescriptorSetLayout, descriptor_pool: vk::DescriptorPool, uniform_buffer: vk::Buffer) -> Vec<vk::DescriptorSet> {
        unsafe {
            let layouts = vec![descriptor_set_layout; 1];
            let info = vk::DescriptorSetAllocateInfo::builder().descriptor_pool(descriptor_pool).set_layouts(&layouts);

            let descriptor_sets = device.allocate_descriptor_sets(&info).unwrap();

            // Range is the size of the EllipseDataStruct
            let info = *vk::DescriptorBufferInfo::builder().buffer(uniform_buffer).offset(0).range(ELLIPSE_DATA_SIZE);

            let buffer_info = &[info];
            let ubo_write = *vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[0])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(buffer_info);

            device.update_descriptor_sets(&[ubo_write], &[] as &[vk::CopyDescriptorSet]);

            return descriptor_sets;
        }
    }

    fn create_uniform_buffer(data: EllipseData, instance: &Instance, device: &Device, physical_device: vk::PhysicalDevice) -> (vk::Buffer, vk::DeviceMemory) {
        unsafe {
            let uniform_buffer_object = data;

            let uniform_buffer_size = ELLIPSE_DATA_SIZE;

            let unfiorm_buffer_info = *vk::BufferCreateInfo::builder()
                .size(uniform_buffer_size)
                .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let uniform_buffer = device.create_buffer(&unfiorm_buffer_info, None).unwrap();

            let uniform_buffer_memory_requirements = device.get_buffer_memory_requirements(uniform_buffer);
            let uniform_buffer_memory_index = get_memory_type_index(
                &instance,
                physical_device,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                uniform_buffer_memory_requirements,
            );

            let uniform_buffer_allocate_info = *vk::MemoryAllocateInfo::builder()
                .allocation_size(uniform_buffer_memory_requirements.size)
                .memory_type_index(uniform_buffer_memory_index);

            let uniform_buffer_memory = device.allocate_memory(&uniform_buffer_allocate_info, None).unwrap();

            let uniform_ptr = device
                .map_memory(uniform_buffer_memory, 0, uniform_buffer_memory_requirements.size, vk::MemoryMapFlags::empty())
                .unwrap();

            let mut align = ash::util::Align::new(uniform_ptr, align_of::<f32>() as u64, ELLIPSE_DATA_SIZE);
            align.copy_from_slice(&[uniform_buffer_object]);

            device.unmap_memory(uniform_buffer_memory);
            device.bind_buffer_memory(uniform_buffer, uniform_buffer_memory, 0).unwrap();

            return (uniform_buffer, uniform_buffer_memory);
        }
    }

    pub fn render(
        &self,
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
        target_image_view: vk::ImageView,
        command_pool: vk::CommandPool,
        graphics_queue: vk::Queue,
        first_element: bool,
        last_element: bool,
        width: u32,
        height: u32,
    ) {
        let command_buffer = create_command_buffer(device, command_pool);

        let render_pass = create_render_pass(device, first_element, last_element);

        let frame_buffer = create_framebuffer(device, target_image_view, render_pass, width, height);

        let vertex_shader = Ellipse::create_vertex_shader(device);
        let fragment_shader = Ellipse::create_fragment_shader(device);

        let viewport = create_viewport(width, height);
        let scissor = create_scissor(width, height);

        let descriptor_set_layouts = Ellipse::create_descriptor_set_layout(device);

        let (graphics_pipeline, graphics_pipeline_layout) = create_graphics_pipeline(
            device,
            vertex_shader,
            fragment_shader,
            viewport,
            scissor,
            render_pass,
            descriptor_set_layouts,
            EllipseVertex::get_descriptor_set_layout_binding(),
            &EllipseVertex::get_attribute_descriptions(),
        );

        let (index_buffer, index_buffer_memory) = create_index_buffer(&vec![0, 1, 2, 2, 3, 0], instance, device, physical_device);

        const X_SCALE: f32 = 1920.0 / 2.0;
        const Y_SCALE: f32 = 1080.0 / 2.0;

        let offsetted_x = self.position.x - self.origin.x * self.size.x;
        let offsetted_y = self.position.y - self.origin.y * self.size.y;

        let (vertex_buffer, vertex_buffer_memory) = Ellipse::create_vertex_buffer(
            &vec![
                vec2(offsetted_x / X_SCALE, -offsetted_y / Y_SCALE),
                vec2(offsetted_x / X_SCALE, -(offsetted_y + self.size.y) / Y_SCALE),
                vec2((offsetted_x + self.size.x) / X_SCALE, -(offsetted_y + self.size.y) / Y_SCALE),
                vec2((offsetted_x + self.size.x) / X_SCALE, -offsetted_y / Y_SCALE),
            ],
            instance,
            device,
            physical_device,
        );

        let (uniform_buffer, uniform_buffer_memory) = Ellipse::create_uniform_buffer(EllipseData { color: self.color }, instance, device, physical_device);

        let descriptor_pools = Ellipse::create_descriptor_pool(device);
        let descriptor_sets = Ellipse::create_descriptor_sets(device, descriptor_set_layouts, descriptor_pools, uniform_buffer);

        begin_render_pass(device, render_pass, frame_buffer, command_buffer, graphics_pipeline, viewport, scissor, width, height);

        unsafe {
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer], &[0]);
            device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT32);
            device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline_layout, 0, &descriptor_sets, &[]);
            device.cmd_draw_indexed(command_buffer, 6, 1, 0, 0, 1);
        }

        end_render_pass(device, command_buffer, graphics_queue);

        unsafe {
            device.destroy_shader_module(vertex_shader, None);
            device.destroy_shader_module(fragment_shader, None);

            device.destroy_descriptor_pool(descriptor_pools, None);

            device.destroy_framebuffer(frame_buffer, None);

            device.destroy_pipeline(graphics_pipeline, None);
            device.destroy_pipeline_layout(graphics_pipeline_layout, None);

            device.destroy_render_pass(render_pass, None);

            device.destroy_descriptor_set_layout(descriptor_set_layouts, None);

            device.destroy_buffer(index_buffer, None);
            device.free_memory(index_buffer_memory, None);

            device.destroy_buffer(vertex_buffer, None);
            device.free_memory(vertex_buffer_memory, None);

            device.destroy_buffer(uniform_buffer, None);
            device.free_memory(uniform_buffer_memory, None);
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

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ClipVertex {
    position: Vector2<f32>,
    uv: Vector2<f32>,
}
const CLIP_VERTEX_SIZE: u64 = 8 + 8;

impl ClipVertex {
    pub fn get_descriptor_set_layout_binding() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(CLIP_VERTEX_SIZE as u32)
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

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct ClipData {
    color: Vector4<f32>,
    size: Vector2<f32>,
}
const CLIP_DATA_SIZE: u64 = 16 + 8;

impl ClipData {
    fn get_descriptor_set_layout_bindings() -> [vk::DescriptorSetLayoutBinding; 1] {
        let layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build();
        [layout_binding]
    }
}

impl Clip {
    fn create_texture_image(
        bytes: Vec<u8>,
        width: u32,
        height: u32,
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        graphics_queue: vk::Queue,
    ) -> (vk::Image, vk::ImageView, vk::DeviceMemory, vk::Sampler) {
        unsafe {
            let staging_buffer_info = *vk::BufferCreateInfo::builder()
                .size(bytes.len() as u64)
                .usage(vk::BufferUsageFlags::TRANSFER_SRC)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let staging_buffer = device.create_buffer(&staging_buffer_info, None).unwrap();

            let staging_buffer_memory_requirements = device.get_buffer_memory_requirements(staging_buffer);
            let staging_buffer_memory_index = get_memory_type_index(
                &instance,
                physical_device,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                staging_buffer_memory_requirements,
            );

            let staging_buffer_allocate_info = *vk::MemoryAllocateInfo::builder()
                .allocation_size(staging_buffer_memory_requirements.size)
                .memory_type_index(staging_buffer_memory_index);

            let staging_buffer_memory = device.allocate_memory(&staging_buffer_allocate_info, None).unwrap();

            let ptr = device
                .map_memory(staging_buffer_memory, 0, staging_buffer_memory_requirements.size, vk::MemoryMapFlags::empty())
                .unwrap();

            copy_nonoverlapping(bytes.as_ptr(), ptr.cast(), bytes.len());

            device.unmap_memory(staging_buffer_memory);
            device.bind_buffer_memory(staging_buffer, staging_buffer_memory, 0).unwrap();

            let texture_image_create_info = vk::ImageCreateInfo::builder()
                .image_type(vk::ImageType::TYPE_2D)
                .format(vk::Format::R8G8B8A8_UNORM)
                .extent(*vk::Extent3D::builder().width(width).height(height).depth(1))
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let texture_image = device.create_image(&texture_image_create_info, None).unwrap();

            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);

            let texture_image_view_create_info = vk::ImageViewCreateInfo::builder()
                .image(texture_image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::R8G8B8A8_UNORM)
                .subresource_range(*subresource_range);

            let texture_image_requirements = device.get_image_memory_requirements(texture_image);

            let texture_image_memory_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(texture_image_requirements.size)
                .memory_type_index(get_memory_type_index(&instance, physical_device, vk::MemoryPropertyFlags::DEVICE_LOCAL, texture_image_requirements));

            let texture_image_memory = device.allocate_memory(&texture_image_memory_info, None).unwrap();
            device.bind_image_memory(texture_image, texture_image_memory, 0).unwrap();

            let texture_image_view = device.create_image_view(&texture_image_view_create_info, None).unwrap();

            transition_image_layout(
                device,
                texture_image,
                vk::Format::R8G8B8A8_SRGB,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                command_pool,
                graphics_queue,
            );

            copy_buffer_to_image(device, staging_buffer, texture_image, width, height, command_pool, graphics_queue);

            transition_image_layout(
                device,
                texture_image,
                vk::Format::R8G8B8A8_SRGB,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                command_pool,
                graphics_queue,
            );

            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);

            let info = vk::SamplerCreateInfo::builder()
                .mag_filter(vk::Filter::LINEAR)
                .min_filter(vk::Filter::LINEAR)
                .address_mode_u(vk::SamplerAddressMode::REPEAT)
                .address_mode_v(vk::SamplerAddressMode::REPEAT)
                .address_mode_w(vk::SamplerAddressMode::REPEAT)
                .anisotropy_enable(false)
                .max_anisotropy(16.0)
                .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
                .unnormalized_coordinates(false)
                .compare_enable(false)
                .compare_op(vk::CompareOp::ALWAYS)
                .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
                .mip_lod_bias(0.0)
                .min_lod(0.0)
                .max_lod(0.0);

            let sampler = device.create_sampler(&info, None).unwrap();

            return (texture_image, texture_image_view, texture_image_memory, sampler);
        }
    }

    fn create_vertex_shader(device: &Device) -> ShaderModule {
        unsafe {
            let mut vertex_spv_file = Cursor::new(&include_bytes!("./shaders/compiled/clip.vert.spv"));

            let vertex_code = read_spv(&mut vertex_spv_file).expect("Failed to read vertex shader spv file");
            let vertex_shader_info = vk::ShaderModuleCreateInfo::builder().code(&vertex_code);

            device.create_shader_module(&vertex_shader_info, None).expect("Vertex shader module error")
        }
    }

    fn create_fragment_shader(device: &Device) -> ShaderModule {
        unsafe {
            let mut frag_spv_file = Cursor::new(&include_bytes!("./shaders/compiled/clip.frag.spv"));

            let frag_code = read_spv(&mut frag_spv_file).expect("Failed to read fragment shader spv file");
            let frag_shader_info = vk::ShaderModuleCreateInfo::builder().code(&frag_code);

            device.create_shader_module(&frag_shader_info, None).expect("Fragment shader module error")
        }
    }

    fn create_vertex_buffer(vertex_positions: &Vec<Vector2<f32>>, instance: &Instance, device: &Device, physical_device: vk::PhysicalDevice) -> (vk::Buffer, vk::DeviceMemory) {
        unsafe {
            let mut vertices: Vec<ClipVertex> = Vec::new();

            for index in 0..(vertex_positions.len()) {
                vertices.push(ClipVertex {
                    position: vertex_positions[index],
                    uv: UVS[index],
                });
            }

            let vertex_buffer_size = CLIP_VERTEX_SIZE * vertices.len() as u64;

            let vertex_buffer_info = *vk::BufferCreateInfo::builder()
                .size(vertex_buffer_size)
                .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let vertex_buffer = device.create_buffer(&vertex_buffer_info, None).unwrap();

            let vertex_buffer_memory_requirements = device.get_buffer_memory_requirements(vertex_buffer);
            let vertex_buffer_memory_index = get_memory_type_index(
                &instance,
                physical_device,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                vertex_buffer_memory_requirements,
            );

            let vertex_buffer_allocate_info = *vk::MemoryAllocateInfo::builder()
                .allocation_size(vertex_buffer_memory_requirements.size)
                .memory_type_index(vertex_buffer_memory_index);

            let vertex_buffer_memory = device.allocate_memory(&vertex_buffer_allocate_info, None).unwrap();

            let vert_ptr = device.map_memory(vertex_buffer_memory, 0, vertex_buffer_memory_requirements.size, vk::MemoryMapFlags::empty()).unwrap();

            copy_nonoverlapping(vertices.as_ptr(), vert_ptr.cast(), vertices.len());

            device.unmap_memory(vertex_buffer_memory);
            device.bind_buffer_memory(vertex_buffer, vertex_buffer_memory, 0).unwrap();

            return (vertex_buffer, vertex_buffer_memory);
        }
    }

    fn create_descriptor_set_layout(device: &Device) -> vk::DescriptorSetLayout {
        let data_bindings = ClipData::get_descriptor_set_layout_bindings();

        let sampler_binding = *vk::DescriptorSetLayoutBinding::builder()
            .binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);

        let mut bindings = Vec::new();
        bindings.extend_from_slice(&data_bindings);
        bindings.push(sampler_binding);

        let layout_info = *vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);

        unsafe { device.create_descriptor_set_layout(&layout_info, None).unwrap() }
    }

    fn create_descriptor_pool(device: &Device) -> vk::DescriptorPool {
        unsafe {
            let uniform_buffer_object_size = *vk::DescriptorPoolSize::builder().ty(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(1);

            let pool_sizes = [uniform_buffer_object_size];
            let info = vk::DescriptorPoolCreateInfo::builder().pool_sizes(&pool_sizes).max_sets(1);

            device.create_descriptor_pool(&info, None).unwrap()
        }
    }

    fn create_descriptor_sets(
        device: &Device,
        descriptor_set_layout: vk::DescriptorSetLayout,
        descriptor_pool: vk::DescriptorPool,
        uniform_buffer: vk::Buffer,
        texture_image_view: vk::ImageView,
        texture_image_sampler: vk::Sampler,
    ) -> Vec<vk::DescriptorSet> {
        unsafe {
            let layouts = vec![descriptor_set_layout; 1];
            let info = vk::DescriptorSetAllocateInfo::builder().descriptor_pool(descriptor_pool).set_layouts(&layouts);

            let descriptor_sets = device.allocate_descriptor_sets(&info).unwrap();

            // Range is the size of the ClipDataStruct
            let info = *vk::DescriptorBufferInfo::builder().buffer(uniform_buffer).offset(0).range(CLIP_DATA_SIZE);

            let buffer_info = &[info];
            let ubo_write = *vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[0])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(buffer_info);

            let info = *vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(texture_image_view)
                .sampler(texture_image_sampler);

            let image_info = &[info];
            let sampler_write = *vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[0])
                .dst_binding(1)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(image_info);

            device.update_descriptor_sets(&[ubo_write, sampler_write], &[] as &[vk::CopyDescriptorSet]);

            return descriptor_sets;
        }
    }

    fn create_uniform_buffer(data: ClipData, instance: &Instance, device: &Device, physical_device: vk::PhysicalDevice) -> (vk::Buffer, vk::DeviceMemory) {
        unsafe {
            let uniform_buffer_object = data;

            let uniform_buffer_size = CLIP_DATA_SIZE;

            let unfiorm_buffer_info = *vk::BufferCreateInfo::builder()
                .size(uniform_buffer_size)
                .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let uniform_buffer = device.create_buffer(&unfiorm_buffer_info, None).unwrap();

            let uniform_buffer_memory_requirements = device.get_buffer_memory_requirements(uniform_buffer);
            let uniform_buffer_memory_index = get_memory_type_index(
                &instance,
                physical_device,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                uniform_buffer_memory_requirements,
            );

            let uniform_buffer_allocate_info = *vk::MemoryAllocateInfo::builder()
                .allocation_size(uniform_buffer_memory_requirements.size)
                .memory_type_index(uniform_buffer_memory_index);

            let uniform_buffer_memory = device.allocate_memory(&uniform_buffer_allocate_info, None).unwrap();

            let uniform_ptr = device
                .map_memory(uniform_buffer_memory, 0, uniform_buffer_memory_requirements.size, vk::MemoryMapFlags::empty())
                .unwrap();

            let mut align = ash::util::Align::new(uniform_ptr, align_of::<f32>() as u64, CLIP_DATA_SIZE);
            align.copy_from_slice(&[uniform_buffer_object]);

            device.unmap_memory(uniform_buffer_memory);
            device.bind_buffer_memory(uniform_buffer, uniform_buffer_memory, 0).unwrap();

            return (uniform_buffer, uniform_buffer_memory);
        }
    }

    pub fn render(
        &self,
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
        target_image_view: vk::ImageView,
        command_pool: vk::CommandPool,
        graphics_queue: vk::Queue,
        first_element: bool,
        last_element: bool,
        width: u32,
        height: u32,
        clip_loader: &ClipLoader,
    ) {
        let mut render_size_x = self.size.x;
        let mut render_size_y = self.size.x;

        let mut clip = clip_loader.get_new(&self.clip).unwrap();

        if let Clips::ImageClip(ref mut clip) = &mut clip {
            render_size_x = clip.width as f32;
            render_size_y = clip.height as f32;
        }

        let mut clip_renderer = Renderer::create(render_size_x as u32, render_size_y as u32);

        let mut render = Vec::new();

        match clip {
            Clips::ScriptClip(mut clip) => {
                clip.set_frame(self.frame);

                render = clip.render(&mut clip_renderer, clip_loader);
            }
            Clips::ImageClip(clip) => {
                render = clip.render(&mut clip_renderer, clip_loader);
            }
        }

        clip_renderer.destroy();

        let (texture_image, texture_image_view, texture_image_memory, texture_image_sampler) =
            Clip::create_texture_image(render, render_size_x as u32, render_size_y as u32, instance, device, physical_device, command_pool, graphics_queue);

        let command_buffer = create_command_buffer(device, command_pool);

        let render_pass = create_render_pass(device, first_element, last_element);

        let frame_buffer = create_framebuffer(device, target_image_view, render_pass, width, height);

        let vertex_shader = Clip::create_vertex_shader(device);
        let fragment_shader = Clip::create_fragment_shader(device);

        let viewport = create_viewport(width, height);
        let scissor = create_scissor(width, height);

        let descriptor_set_layouts = Clip::create_descriptor_set_layout(device);

        let (graphics_pipeline, graphics_pipeline_layout) = create_graphics_pipeline(
            device,
            vertex_shader,
            fragment_shader,
            viewport,
            scissor,
            render_pass,
            descriptor_set_layouts,
            ClipVertex::get_descriptor_set_layout_binding(),
            &ClipVertex::get_attribute_descriptions(),
        );

        let (index_buffer, index_buffer_memory) = create_index_buffer(&vec![0, 1, 2, 2, 3, 0], instance, device, physical_device);

        let normalize_scale = vec2(1920.0 / 2.0, 1080.0 / 2.0);

        let offsetted_x = self.position.x - self.origin.x * self.size.x;
        let offsetted_y = self.position.y - self.origin.y * self.size.y;

        let mut vertices = vec![
            vec2(offsetted_x, offsetted_y),
            vec2(offsetted_x, offsetted_y + self.size.y),
            vec2(offsetted_x + self.size.x, offsetted_y + self.size.y),
            vec2(offsetted_x + self.size.x, offsetted_y),
        ];

        for vertex_index in 0..vertices.len() {
            vertices[vertex_index] = flip_vertically(divide(rotate(vertices[vertex_index], self.position, self.rotation), normalize_scale))
        }

        let (vertex_buffer, vertex_buffer_memory) = Clip::create_vertex_buffer(&vertices, instance, device, physical_device);

        let (uniform_buffer, uniform_buffer_memory) = Clip::create_uniform_buffer(ClipData { color: self.color, size: self.size }, instance, device, physical_device);

        let descriptor_pools = Clip::create_descriptor_pool(device);
        let descriptor_sets = Clip::create_descriptor_sets(device, descriptor_set_layouts, descriptor_pools, uniform_buffer, texture_image_view, texture_image_sampler);

        begin_render_pass(device, render_pass, frame_buffer, command_buffer, graphics_pipeline, viewport, scissor, width, height);

        unsafe {
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer], &[0]);
            device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT32);
            device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline_layout, 0, &descriptor_sets, &[]);
            device.cmd_draw_indexed(command_buffer, 6, 1, 0, 0, 1);
        }

        end_render_pass(device, command_buffer, graphics_queue);

        unsafe {
            device.destroy_shader_module(vertex_shader, None);
            device.destroy_shader_module(fragment_shader, None);

            device.destroy_descriptor_pool(descriptor_pools, None);

            device.destroy_framebuffer(frame_buffer, None);

            device.destroy_pipeline(graphics_pipeline, None);
            device.destroy_pipeline_layout(graphics_pipeline_layout, None);

            device.destroy_render_pass(render_pass, None);

            device.destroy_descriptor_set_layout(descriptor_set_layouts, None);

            device.free_memory(texture_image_memory, None);
            device.destroy_image_view(texture_image_view, None);
            device.destroy_image(texture_image, None);

            device.destroy_buffer(index_buffer, None);
            device.free_memory(index_buffer_memory, None);

            device.destroy_buffer(vertex_buffer, None);
            device.free_memory(vertex_buffer_memory, None);

            device.destroy_buffer(uniform_buffer, None);
            device.free_memory(uniform_buffer_memory, None);
        }
    }
}
