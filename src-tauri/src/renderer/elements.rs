use std::{mem, ptr::copy_nonoverlapping};

use ash::{util::Align, vk, Device, Instance};
use cgmath::{vec2, vec3, Vector2, Vector3};

pub enum Elements {
    Rect(Rect),
}

pub trait Element {
    fn render(&self, command_buffer: vk::CommandBuffer, instance: &Instance, device: &Device, physical_device: vk::PhysicalDevice);
}

pub struct Rect {
    pub position: Vector2<f32>,
    pub size: Vector2<f32>,
}

impl Element for Rect {
    fn render(&self, command_buffer: vk::CommandBuffer, instance: &Instance, device: &Device, physical_device: vk::PhysicalDevice) {
        let index_buffer = create_index_buffer(&vec![0, 1, 2, 2, 3, 0], instance, device, physical_device);
        let vertex_buffer = create_vertex_buffer(&vec![vec2(0.0, 0.0), vec2(0.0, 0.5), vec2(0.5, 0.5), vec2(0.5, 0.0)], instance, device, physical_device);

        unsafe {
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer], &[0]);
            device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT32);
            device.cmd_draw_indexed(command_buffer, 6, 1, 0, 0, 1);
        }
    }
}

const COLORS: [Vector3<f32>; 3] = [vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, 1.0)];

fn create_index_buffer(indices: &Vec<u32>, instance: &Instance, device: &Device, physical_device: vk::PhysicalDevice) -> vk::Buffer {
    unsafe {
        let index_buffer_size = 4 * indices.len() as u64;

        let index_buffer_info = vk::BufferCreateInfo::builder()
            .size(index_buffer_size)
            .usage(vk::BufferUsageFlags::INDEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let index_buffer = device.create_buffer(&index_buffer_info, None).unwrap();

        let index_buffer_memory_requirements = device.get_buffer_memory_requirements(index_buffer);
        let index_buffer_memory_index = get_memory_type_index(
            &instance,
            physical_device,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            index_buffer_memory_requirements,
        );

        let index_allocate_info = *vk::MemoryAllocateInfo::builder()
            .allocation_size(index_buffer_memory_requirements.size)
            .memory_type_index(index_buffer_memory_index);

        let index_buffer_memory = device.allocate_memory(&index_allocate_info, None).unwrap();

        let index_ptr = device.map_memory(index_buffer_memory, 0, index_buffer_memory_requirements.size, vk::MemoryMapFlags::empty()).unwrap();
        let mut index_slice = Align::new(index_ptr, mem::align_of::<u32>() as u64, index_buffer_memory_requirements.size);
        index_slice.copy_from_slice(&indices);

        device.unmap_memory(index_buffer_memory);
        device.bind_buffer_memory(index_buffer, index_buffer_memory, 0).unwrap();

        return index_buffer;
    }
}

fn create_vertex_buffer(vertex_positions: &Vec<Vector2<f32>>, instance: &Instance, device: &Device, physical_device: vk::PhysicalDevice) -> vk::Buffer {
    unsafe {
        let mut vertices: Vec<Vertex> = Vec::new();

        for index in 0..(vertex_positions.len()) {
            vertices.push(Vertex::new(vertex_positions[index], COLORS[index % COLORS.len()]));
        }

        let vertex_buffer_size = 20 * vertices.len() as u64;

        let vertex_buffer_info = *vk::BufferCreateInfo::builder()
            .size((20 * vertices.len()) as u64)
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

        return vertex_buffer;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pos: Vector2<f32>,
    color: Vector3<f32>,
}

impl Vertex {
    const fn new(pos: Vector2<f32>, color: Vector3<f32>) -> Self {
        Self { pos, color }
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder().binding(0).stride(20).input_rate(vk::VertexInputRate::VERTEX).build()
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        let pos = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(0)
            .build();
        let color = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(8)
            .build();

        [pos, color]
    }
}

unsafe fn get_memory_type_index(instance: &Instance, physical_device: vk::PhysicalDevice, properties: vk::MemoryPropertyFlags, requirements: vk::MemoryRequirements) -> u32 {
    let memory = instance.get_physical_device_memory_properties(physical_device);

    (0..memory.memory_type_count)
        .find(|i| {
            let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
            let memory_type = memory.memory_types[*i as usize];

            suitable && memory_type.property_flags.contains(properties)
        })
        .unwrap()
}
