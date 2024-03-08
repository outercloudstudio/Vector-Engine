use ash::{util::Align, vk, Device, Instance};
use std::{ffi::CStr, mem};

pub fn create_command_buffer(device: &Device, command_pool: vk::CommandPool) -> vk::CommandBuffer {
    unsafe {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(1)
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers = device.allocate_command_buffers(&command_buffer_allocate_info).unwrap();

        command_buffers[0]
    }
}

pub fn create_render_pass(device: &Device, first_element: bool, last_element: bool) -> vk::RenderPass {
    unsafe {
        let color_attachment = *vk::AttachmentDescription::builder()
            .format(vk::Format::R8G8B8A8_UNORM)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(if first_element { vk::AttachmentLoadOp::CLEAR } else { vk::AttachmentLoadOp::LOAD })
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(if first_element { vk::ImageLayout::UNDEFINED } else { vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL })
            .final_layout(if last_element {
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL
            } else {
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            });

        let color_attachment_ref = *vk::AttachmentReference::builder().attachment(0).layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let subpass = *vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&[color_attachment_ref]);

        let dependency = *vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

        let render_pass_attachments = &[color_attachment];
        let subpasses = &[subpass];
        let dependencies = &[dependency];

        let render_pass_create_info = vk::RenderPassCreateInfo::builder().attachments(render_pass_attachments).subpasses(subpasses).dependencies(dependencies);

        device.create_render_pass(&render_pass_create_info, None).unwrap()
    }
}

pub fn create_framebuffer(device: &Device, target_image_view: vk::ImageView, render_pass: vk::RenderPass, width: u32, height: u32) -> vk::Framebuffer {
    unsafe {
        let frame_buffer_attachments = &[target_image_view];

        let frame_buffer_create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(frame_buffer_attachments)
            .width(width)
            .height(height)
            .layers(1);

        device.create_framebuffer(&frame_buffer_create_info, None).unwrap()
    }
}

pub fn create_viewport(width: u32, height: u32) -> vk::Viewport {
    vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: width as f32,
        height: height as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    }
}

pub fn create_scissor(width: u32, height: u32) -> vk::Rect2D {
    *vk::Rect2D::builder().extent(*vk::Extent2D::builder().width(width).height(height))
}

pub fn create_graphics_pipeline(
    device: &Device,
    vertex_shader: vk::ShaderModule,
    fragment_shader: vk::ShaderModule,
    viewport: vk::Viewport,
    scissor: vk::Rect2D,
    render_pass: vk::RenderPass,
    descriptor_set_layout: vk::DescriptorSetLayout,
    binding_description: vk::VertexInputBindingDescription,
    attribute_description: &[vk::VertexInputAttributeDescription],
) -> (vk::Pipeline, vk::PipelineLayout) {
    unsafe {
        let layouts = [descriptor_set_layout];
        let layout_create_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(&layouts);

        let pipeline_layout = device.create_pipeline_layout(&layout_create_info, None).unwrap();

        let shader_entry_name = CStr::from_bytes_with_nul_unchecked(b"main\0");
        let shader_stage_create_infos = [
            vk::PipelineShaderStageCreateInfo {
                module: vertex_shader,
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                module: fragment_shader,
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];

        let binding_descriptions = &[binding_description];
        let vertex_input_state = *vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(binding_descriptions)
            .vertex_attribute_descriptions(attribute_description);

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewports = &[viewport];

        let scissors = &[scissor];

        let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder().scissors(scissors).viewports(viewports);

        let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            line_width: 1.0,
            polygon_mode: vk::PolygonMode::FILL,
            ..Default::default()
        };

        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };

        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
            blend_enable: vk::TRUE,
            src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::SRC_ALPHA,
            dst_alpha_blend_factor: vk::BlendFactor::DST_ALPHA,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        }];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op(vk::LogicOp::CLEAR)
            .attachments(&color_blend_attachment_states);

        let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);

        let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stage_create_infos)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_state_info)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state_info)
            .layout(pipeline_layout)
            .render_pass(render_pass);

        let graphics_pipelines = device
            .create_graphics_pipelines(vk::PipelineCache::null(), &[graphic_pipeline_info.build()], None)
            .expect("Unable to create graphics pipeline");

        (graphics_pipelines[0], pipeline_layout)
    }
}

pub fn create_index_buffer(indices: &Vec<u32>, instance: &Instance, device: &Device, physical_device: vk::PhysicalDevice) -> (vk::Buffer, vk::DeviceMemory) {
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

        return (index_buffer, index_buffer_memory);
    }
}

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

pub fn begin_render_pass(
    device: &Device,
    render_pass: vk::RenderPass,
    frame_buffer: vk::Framebuffer,
    command_buffer: vk::CommandBuffer,
    graphics_pipeline: vk::Pipeline,
    viewport: vk::Viewport,
    scissor: vk::Rect2D,
    width: u32,
    height: u32,
) {
    unsafe {
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 0.0] },
        }];

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass)
            .framebuffer(frame_buffer)
            .render_area(*vk::Rect2D::builder().extent(*vk::Extent2D::builder().width(width).height(height)))
            .clear_values(&clear_values);

        device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::RELEASE_RESOURCES).unwrap();

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        device.begin_command_buffer(command_buffer, &command_buffer_begin_info).expect("Begin commandbuffer");

        device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
        device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline);
        device.cmd_set_viewport(command_buffer, 0, &[viewport]);
        device.cmd_set_scissor(command_buffer, 0, &[scissor]);
    }
}

pub fn end_render_pass(device: &Device, command_buffer: vk::CommandBuffer, graphics_queue: vk::Queue) {
    unsafe {
        device.cmd_end_render_pass(command_buffer);

        device.end_command_buffer(command_buffer).expect("End commandbuffer");

        let command_buffers = vec![command_buffer];

        let mut submit_info = vk::SubmitInfo::builder()
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(&command_buffers);
        submit_info.wait_semaphore_count = 0;

        device.queue_submit(graphics_queue, &[submit_info.build()], vk::Fence::null()).expect("queue submit failed.");

        device.queue_wait_idle(graphics_queue).unwrap();
    }
}

fn begin_single_time_commands(device: &Device, command_pool: vk::CommandPool) -> vk::CommandBuffer {
    unsafe {
        let info = vk::CommandBufferAllocateInfo::builder()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(command_pool)
            .command_buffer_count(1);

        let command_buffer = device.allocate_command_buffers(&info).unwrap()[0];

        let info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        device.begin_command_buffer(command_buffer, &info).unwrap();

        return command_buffer;
    }
}

fn end_single_time_commands(device: &Device, command_buffer: vk::CommandBuffer, graphics_queue: vk::Queue, command_pool: vk::CommandPool) {
    unsafe {
        device.end_command_buffer(command_buffer).unwrap();

        let command_buffers = &[command_buffer];
        let info = *vk::SubmitInfo::builder().command_buffers(command_buffers);

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

        let subresource = *vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);

        let barrier = *vk::ImageMemoryBarrier::builder()
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

        let subresource = *vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1);

        let region = *vk::BufferImageCopy::builder()
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
