#![allow(dead_code, unused_variables)]

use cgmath::{point3, vec2, vec3, Deg};
use log::*;
use std::ffi::{c_char, CStr};
use std::io::Cursor;
use std::mem;
use std::ptr::copy_nonoverlapping;
use std::{borrow::Cow, default::Default};

use ash::extensions::ext::DebugUtils;
use ash::util::*;
use ash::{vk, Device, Entry, Instance};

type Vec2 = cgmath::Vector2<f32>;
type Vec3 = cgmath::Vector3<f32>;
type Mat4 = cgmath::Matrix4<f32>;

pub struct Renderer {
    instance: Instance,
    device: Device,
    command_buffer: vk::CommandBuffer,
    graphics_queue: vk::Queue,
    render_pass: vk::RenderPass,
    frame_buffer: vk::Framebuffer,
    physical_device: vk::PhysicalDevice,
    command_pool: vk::CommandPool,
    graphics_pipeline: vk::Pipeline,
    target_image: vk::Image,
    scissor: vk::Rect2D,
    viewport: vk::Viewport,
}

impl Renderer {
    pub fn create() -> Renderer {
        unsafe {
            let entry = Entry::linked();

            let layer_names = [CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0")];
            let layers_names_raw: Vec<*const c_char> = layer_names.iter().map(|raw_name| raw_name.as_ptr()).collect();

            let extension_names = vec![DebugUtils::name().as_ptr()];

            let appinfo = vk::ApplicationInfo::builder()
                .application_name(CStr::from_bytes_with_nul_unchecked(b"VulkanTriangle\0"))
                .application_version(0)
                .engine_name(CStr::from_bytes_with_nul_unchecked(b"VulkanTriangle\0"))
                .engine_version(0)
                .api_version(vk::make_api_version(0, 1, 0, 0));

            let create_info = vk::InstanceCreateInfo::builder()
                .application_info(&appinfo)
                .enabled_layer_names(&layers_names_raw)
                .enabled_extension_names(&extension_names);

            let instance: Instance = entry.create_instance(&create_info, None).expect("Instance creation error");

            let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | vk::DebugUtilsMessageSeverityFlagsEXT::INFO)
                .message_type(vk::DebugUtilsMessageTypeFlagsEXT::GENERAL | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE)
                .pfn_user_callback(Some(vulkan_debug_callback));

            let debug_utils_loader = DebugUtils::new(&entry, &instance);

            let debug_call_back = debug_utils_loader.create_debug_utils_messenger(&debug_info, None).unwrap();

            let pdevices = instance.enumerate_physical_devices().expect("Physical device error");

            let (pdevice, queue_family_index) = pdevices
                .iter()
                .find_map(|pdevice| {
                    instance.get_physical_device_queue_family_properties(*pdevice).iter().enumerate().find_map(|(index, info)| {
                        if info.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                            Some((*pdevice, index))
                        } else {
                            None
                        }
                    })
                })
                .expect("Couldn't find suitable device.");

            let queue_family_index = queue_family_index as u32;

            let features = vk::PhysicalDeviceFeatures {
                shader_clip_distance: 1,
                ..Default::default()
            };
            let priorities = [1.0];

            let queue_info = vk::DeviceQueueCreateInfo::builder().queue_family_index(queue_family_index).queue_priorities(&priorities);

            let device_create_info = vk::DeviceCreateInfo::builder().queue_create_infos(std::slice::from_ref(&queue_info)).enabled_features(&features);

            let device: Device = instance.create_device(pdevice, &device_create_info, None).unwrap();

            let graphics_queue = device.get_device_queue(queue_family_index, 0);

            let pool_create_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(queue_family_index);

            let pool = device.create_command_pool(&pool_create_info, None).unwrap();

            let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_buffer_count(1)
                .command_pool(pool)
                .level(vk::CommandBufferLevel::PRIMARY);

            let command_buffers = device.allocate_command_buffers(&command_buffer_allocate_info).unwrap();

            let command_buffer = command_buffers[0];

            let device_memory_properties = instance.get_physical_device_memory_properties(pdevice);

            let target_image_create_info = vk::ImageCreateInfo::builder()
                .image_type(vk::ImageType::TYPE_2D)
                .format(vk::Format::R8G8B8A8_SRGB)
                .extent(*vk::Extent3D::builder().width(512).height(512).depth(1))
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let target_image = device.create_image(&target_image_create_info, None).unwrap();

            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);

            let target_image_view_create_info = vk::ImageViewCreateInfo::builder()
                .image(target_image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::R8G8B8A8_SRGB)
                .subresource_range(*subresource_range);

            let target_image_requirements = device.get_image_memory_requirements(target_image);

            let target_image_memory_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(target_image_requirements.size)
                .memory_type_index(get_memory_type_index(&instance, pdevice, vk::MemoryPropertyFlags::DEVICE_LOCAL, target_image_requirements));

            let target_image_memory = device.allocate_memory(&target_image_memory_info, None).unwrap();
            device.bind_image_memory(target_image, target_image_memory, 0).unwrap();

            let target_image_view = device.create_image_view(&target_image_view_create_info, None).unwrap();

            let color_attachment = *vk::AttachmentDescription::builder()
                .format(vk::Format::R8G8B8A8_SRGB)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL);

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

            let render_pass = device.create_render_pass(&render_pass_create_info, None).unwrap();

            let frame_buffer_attachments = &[target_image_view];

            let frame_buffer_create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(frame_buffer_attachments)
                .width(512)
                .height(512)
                .layers(1);

            let frame_buffer = device.create_framebuffer(&frame_buffer_create_info, None).unwrap();

            let mut vertex_spv_file = Cursor::new(&include_bytes!("./shaders/vert.spv"));
            let mut frag_spv_file = Cursor::new(&include_bytes!("./shaders/frag.spv"));

            let vertex_code = read_spv(&mut vertex_spv_file).expect("Failed to read vertex shader spv file");
            let vertex_shader_info = vk::ShaderModuleCreateInfo::builder().code(&vertex_code);

            let frag_code = read_spv(&mut frag_spv_file).expect("Failed to read fragment shader spv file");
            let frag_shader_info = vk::ShaderModuleCreateInfo::builder().code(&frag_code);

            let vertex_shader_module = device.create_shader_module(&vertex_shader_info, None).expect("Vertex shader module error");
            let fragment_shader_module = device.create_shader_module(&frag_shader_info, None).expect("Fragment shader module error");

            let layout_create_info = vk::PipelineLayoutCreateInfo::default();

            let pipeline_layout = device.create_pipeline_layout(&layout_create_info, None).unwrap();

            let shader_entry_name = CStr::from_bytes_with_nul_unchecked(b"main\0");
            let shader_stage_create_infos = [
                vk::PipelineShaderStageCreateInfo {
                    module: vertex_shader_module,
                    p_name: shader_entry_name.as_ptr(),
                    stage: vk::ShaderStageFlags::VERTEX,
                    ..Default::default()
                },
                vk::PipelineShaderStageCreateInfo {
                    module: fragment_shader_module,
                    p_name: shader_entry_name.as_ptr(),
                    stage: vk::ShaderStageFlags::FRAGMENT,
                    ..Default::default()
                },
            ];

            let binding_descriptions = &[Vertex::binding_description()];
            let attribute_descriptions = Vertex::attribute_descriptions();
            let vertex_input_state = *vk::PipelineVertexInputStateCreateInfo::builder()
                .vertex_binding_descriptions(binding_descriptions)
                .vertex_attribute_descriptions(&attribute_descriptions);

            let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                .primitive_restart_enable(false);

            let viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: 512.0,
                height: 512.0,
                min_depth: 0.0,
                max_depth: 1.0,
            };
            let viewports = &[viewport];

            let scissor = *vk::Rect2D::builder().extent(*vk::Extent2D::builder().width(512).height(512));
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
                blend_enable: 0,
                src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
                dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
                color_blend_op: vk::BlendOp::ADD,
                src_alpha_blend_factor: vk::BlendFactor::ZERO,
                dst_alpha_blend_factor: vk::BlendFactor::ZERO,
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

            let graphic_pipeline = graphics_pipelines[0];

            Renderer {
                instance,
                device,
                command_buffer,
                graphics_queue,
                render_pass,
                frame_buffer,
                physical_device: pdevice,
                command_pool: pool,
                graphics_pipeline: graphic_pipeline,
                target_image,
                scissor,
                viewport,
            }
        }
    }

    pub fn render(self: &Renderer, vertex_data: Vec<f32>, indices: Vec<u32>) -> Vec<u8> {
        let instance = &self.instance;
        let device = &self.device;
        let command_buffer = self.command_buffer;
        let graphics_queue = self.graphics_queue;
        let render_pass = self.render_pass;
        let frame_buffer = self.frame_buffer;
        let graphic_pipeline = self.graphics_pipeline;
        let pdevice = self.physical_device;
        let pool = self.command_pool;
        let target_image = self.target_image;
        let scissor = self.scissor;
        let viewport = self.viewport;

        unsafe {
            info!("Indices: {}", indices.iter().map(|index| index.to_string()).collect::<Vec<String>>().join(" "));

            let index_buffer_info = vk::BufferCreateInfo::builder()
                .size(4 * indices.len() as u64)
                .usage(vk::BufferUsageFlags::INDEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let index_buffer = device.create_buffer(&index_buffer_info, None).unwrap();

            let index_buffer_memory_req = device.get_buffer_memory_requirements(index_buffer);
            let index_buffer_memory_index = get_memory_type_index(
                &instance,
                pdevice,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                index_buffer_memory_req,
            );

            let index_allocate_info = *vk::MemoryAllocateInfo::builder()
                .allocation_size(index_buffer_memory_req.size)
                .memory_type_index(index_buffer_memory_index);

            let index_buffer_memory = device.allocate_memory(&index_allocate_info, None).unwrap();

            info!("Size {}", index_buffer_memory_req.size);

            let index_ptr = device.map_memory(index_buffer_memory, 0, index_buffer_memory_req.size, vk::MemoryMapFlags::empty()).unwrap();
            let mut index_slice = Align::new(index_ptr, mem::align_of::<u32>() as u64, index_buffer_memory_req.size);
            index_slice.copy_from_slice(&indices);

            device.unmap_memory(index_buffer_memory);
            device.bind_buffer_memory(index_buffer, index_buffer_memory, 0).unwrap();

            const COLORS: [Vec3; 3] = [vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, 1.0)];

            let mut vertices: Vec<Vertex> = Vec::new();

            for i in 0..(vertex_data.len() / 2) {
                vertices.push(Vertex::new(vec2(vertex_data[i * 2], vertex_data[i * 2 + 1]), COLORS[i % COLORS.len()]));
            }

            info!(
                "Vertices: {}",
                vertices
                    .iter()
                    .map(|vertex| format!("{} {}", vertex.pos.x.to_string(), vertex.pos.y.to_string()))
                    .collect::<Vec<String>>()
                    .join("\n")
            );

            let vertex_input_buffer_info = *vk::BufferCreateInfo::builder()
                .size((20 * vertices.len()) as u64)
                .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let vertex_input_buffer = device.create_buffer(&vertex_input_buffer_info, None).unwrap();

            let vertex_input_buffer_memory_req = device.get_buffer_memory_requirements(vertex_input_buffer);
            let vertex_input_buffer_memory_index = get_memory_type_index(
                &instance,
                pdevice,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                vertex_input_buffer_memory_req,
            );

            let vertex_buffer_allocate_info = *vk::MemoryAllocateInfo::builder()
                .allocation_size(vertex_input_buffer_memory_req.size)
                .memory_type_index(vertex_input_buffer_memory_index);

            let vertex_input_buffer_memory = device.allocate_memory(&vertex_buffer_allocate_info, None).unwrap();

            let vert_ptr = device
                .map_memory(vertex_input_buffer_memory, 0, vertex_input_buffer_memory_req.size, vk::MemoryMapFlags::empty())
                .unwrap();

            copy_nonoverlapping(vertices.as_ptr(), vert_ptr.cast(), vertices.len());

            device.unmap_memory(vertex_input_buffer_memory);
            device.bind_buffer_memory(vertex_input_buffer, vertex_input_buffer_memory, 0).unwrap();

            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue { float32: [0.0001, 0.0, 0.01, 1.0] },
            }];

            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(render_pass)
                .framebuffer(frame_buffer)
                .render_area(*vk::Rect2D::builder().extent(*vk::Extent2D::builder().width(512).height(512)))
                .clear_values(&clear_values);

            device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::RELEASE_RESOURCES).unwrap();

            let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            device.begin_command_buffer(command_buffer, &command_buffer_begin_info).expect("Begin commandbuffer");

            device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, graphic_pipeline);
            device.cmd_set_viewport(command_buffer, 0, &[viewport]);
            device.cmd_set_scissor(command_buffer, 0, &[scissor]);
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_input_buffer], &[0]);
            device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT32);
            device.cmd_draw_indexed(command_buffer, indices.len() as u32, 1, 0, 0, 1);
            device.cmd_end_render_pass(command_buffer);

            device.end_command_buffer(command_buffer).expect("End commandbuffer");

            let command_buffers = vec![command_buffer];

            let mut submit_info = vk::SubmitInfo::builder()
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .command_buffers(&command_buffers);
            submit_info.wait_semaphore_count = 0;

            device.queue_submit(graphics_queue, &[submit_info.build()], vk::Fence::null()).expect("queue submit failed.");

            device.queue_wait_idle(graphics_queue).unwrap();

            {
                let size = 512 * 512 * 4;

                let save_buffer_info = vk::BufferCreateInfo::builder()
                    .size(size)
                    .usage(vk::BufferUsageFlags::TRANSFER_DST)
                    .sharing_mode(vk::SharingMode::EXCLUSIVE);

                let save_buffer = device.create_buffer(&save_buffer_info, None).unwrap();

                let save_buffer_memory_req = device.get_buffer_memory_requirements(save_buffer);
                let save_buffer_memory_index = get_memory_type_index(
                    &instance,
                    pdevice,
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                    save_buffer_memory_req,
                );

                let index_allocate_info = *vk::MemoryAllocateInfo::builder()
                    .allocation_size(save_buffer_memory_req.size)
                    .memory_type_index(save_buffer_memory_index);

                let save_buffer_memory = device.allocate_memory(&index_allocate_info, None).unwrap();

                device.bind_buffer_memory(save_buffer, save_buffer_memory, 0).unwrap();

                let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
                    .level(vk::CommandBufferLevel::PRIMARY)
                    .command_pool(pool)
                    .command_buffer_count(1);

                let command_buffer = device.allocate_command_buffers(&command_buffer_allocate_info).unwrap()[0];

                let info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

                device.begin_command_buffer(command_buffer, &info).unwrap();

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
                    .image_extent(vk::Extent3D { width: 512, height: 512, depth: 1 });

                device.cmd_copy_image_to_buffer(command_buffer, target_image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, save_buffer, &[region]);

                device.end_command_buffer(command_buffer).unwrap();

                let command_buffers = &[command_buffer];
                let info = *vk::SubmitInfo::builder().command_buffers(command_buffers);

                device.queue_submit(graphics_queue, &[info], vk::Fence::null()).unwrap();
                device.queue_wait_idle(graphics_queue).unwrap();

                device.free_command_buffers(pool, &[command_buffer]);

                let memory = device.map_memory(save_buffer_memory, 0, size, vk::MemoryMapFlags::empty()).unwrap();

                let mut pixels = vec![0; size as usize];

                copy_nonoverlapping(memory.cast(), pixels.as_mut_ptr(), size as usize);

                device.unmap_memory(save_buffer_memory);

                device.destroy_buffer(save_buffer, None);
                device.free_memory(save_buffer_memory, None);

                return pixels;
            }
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    pos: Vec2,
    color: Vec3,
}

impl Vertex {
    const fn new(pos: Vec2, color: Vec3) -> Self {
        Self { pos, color }
    }

    fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder().binding(0).stride(20).input_rate(vk::VertexInputRate::VERTEX).build()
    }

    fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
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

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    // println!("{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n",);

    vk::FALSE
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
