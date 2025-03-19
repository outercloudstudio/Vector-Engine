use ash::{vk, Device, Instance};

pub unsafe fn get_memory_type_index(instance: &Instance, physical_device: vk::PhysicalDevice, properties: vk::MemoryPropertyFlags, requirements: vk::MemoryRequirements) -> u32 {
    let memory = instance.get_physical_device_memory_properties(physical_device);

    (0..memory.memory_type_count)
        .find(|i| {
            let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
            let memory_type = memory.memory_types[*i as usize];

            suitable && memory_type.property_flags.contains(properties)
        })
        .unwrap()
}

fn begin_single_time_commands(device: &Device, command_pool: vk::CommandPool) -> vk::CommandBuffer {
    unsafe {
        let info = vk::CommandBufferAllocateInfo::default()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(command_pool)
            .command_buffer_count(1);

        let command_buffer = device.allocate_command_buffers(&info).unwrap()[0];

        let info = vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        device.begin_command_buffer(command_buffer, &info).unwrap();

        return command_buffer;
    }
}

fn end_single_time_commands(device: &Device, command_buffer: vk::CommandBuffer, graphics_queue: vk::Queue, command_pool: vk::CommandPool) {
    unsafe {
        device.end_command_buffer(command_buffer).unwrap();

        let command_buffers = &[command_buffer];
        let info = vk::SubmitInfo::default().command_buffers(command_buffers);

        device.queue_submit(graphics_queue, &[info], vk::Fence::null()).unwrap();
        device.queue_wait_idle(graphics_queue).unwrap();

        device.free_command_buffers(command_pool, &[command_buffer]);
    }
}

pub fn transition_image_layout(
    device: &Device,
    image: vk::Image,
    format: vk::Format,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    command_pool: vk::CommandPool,
    graphics_queue: vk::Queue,
) {
    unsafe {
        let (src_access_mask, dst_access_mask, src_stage_mask, dst_stage_mask) = match (old_layout, new_layout) {
            (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                vk::AccessFlags::empty(),
                vk::AccessFlags::TRANSFER_WRITE,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
            ),
            (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
                vk::AccessFlags::TRANSFER_WRITE,
                vk::AccessFlags::SHADER_READ,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
            ),
            _ => {
                return;
            }
        };

        let command_buffer = begin_single_time_commands(device, command_pool);

        let subresource = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);

        let barrier = vk::ImageMemoryBarrier::default()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(subresource)
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask);

        device.cmd_pipeline_barrier(
            command_buffer,
            src_stage_mask,
            dst_stage_mask,
            vk::DependencyFlags::empty(),
            &[] as &[vk::MemoryBarrier],
            &[] as &[vk::BufferMemoryBarrier],
            &[barrier],
        );

        end_single_time_commands(device, command_buffer, graphics_queue, command_pool);
    }
}

pub fn copy_buffer_to_image(device: &Device, buffer: vk::Buffer, image: vk::Image, width: u32, height: u32, command_pool: vk::CommandPool, graphics_queue: vk::Queue) {
    unsafe {
        let command_buffer = begin_single_time_commands(device, command_pool);

        let subresource = vk::ImageSubresourceLayers::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1);

        let region = vk::BufferImageCopy::default()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(subresource)
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(vk::Extent3D { width, height, depth: 1 });

        device.cmd_copy_buffer_to_image(command_buffer, buffer, image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[region]);

        end_single_time_commands(device, command_buffer, graphics_queue, command_pool);
    }
}
