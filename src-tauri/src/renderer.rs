#![allow(dead_code, unused_variables)]

use ash::vk::Handle;
use log::*;
use std::env;
use std::ffi::{c_char, CStr};
use std::io::Cursor;
use std::mem;
use std::{borrow::Cow, default::Default};

use ash::extensions::{
    ext::DebugUtils,
    khr::{Surface, Swapchain},
};
use ash::util::*;
use ash::{vk, Device, Entry, Instance};

pub struct Renderer {
    // pub entry: Entry,
    // pub instance: Instance,
    // pub device: Device,
    // pub debug_utils_loader: DebugUtils,
    // pub debug_call_back: vk::DebugUtilsMessengerEXT,

    // pub pdevice: vk::PhysicalDevice,
    // pub device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    // pub queue_family_index: u32,

    // pub target_image: vk::Image,
    // pub target_image_view: vk::ImageView,

    // pub pool: vk::CommandPool,
    // pub draw_command_buffer: vk::CommandBuffer,
    // pub setup_command_buffer: vk::CommandBuffer,
}

impl Renderer {
    pub fn create() -> Renderer {
        unsafe {
            let entry = Entry::linked();

            let layer_names = [CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0")];
            let layers_names_raw: Vec<*const c_char> = layer_names.iter().map(|raw_name| raw_name.as_ptr()).collect();

            let mut extension_names = vec![DebugUtils::name().as_ptr()];

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

            let pool_create_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(queue_family_index);

            let pool = device.create_command_pool(&pool_create_info, None).unwrap();

            let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_buffer_count(2)
                .command_pool(pool)
                .level(vk::CommandBufferLevel::PRIMARY);

            let command_buffers = device.allocate_command_buffers(&command_buffer_allocate_info).unwrap();

            let setup_command_buffer = command_buffers[0];
            let draw_command_buffer = command_buffers[1];

            let device_memory_properties = instance.get_physical_device_memory_properties(pdevice);

            let target_image_create_info = vk::ImageCreateInfo::builder()
                .image_type(vk::ImageType::TYPE_2D)
                .format(vk::Format::R8G8B8A8_SRGB)
                .extent(*vk::Extent3D::builder().width(512).height(512))
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

            let target_image_view = device.create_image_view(&target_image_view_create_info, None).unwrap();

            let target_image_requirements = device.get_image_memory_requirements(target_image);

            let target_image_memory_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(target_image_requirements.size)
                .memory_type_index(get_memory_type_index(&instance, pdevice, vk::MemoryPropertyFlags::DEVICE_LOCAL, target_image_requirements));

            let target_image_memory = device.allocate_memory(&target_image_memory_info, None).unwrap();
            device.bind_image_memory(target_image, target_image_memory, 0).unwrap();

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

            let frame_buffer_create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(&[target_image_view])
                .width(512)
                .height(512)
                .layers(1);

            let index_buffer_data = [0u32, 1, 2];

            let index_buffer_info = vk::BufferCreateInfo::builder()
                .size(std::mem::size_of_val(&index_buffer_data) as u64)
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

            let index_ptr = device.map_memory(index_buffer_memory, 0, index_buffer_memory_req.size, vk::MemoryMapFlags::empty()).unwrap();
            let mut index_slice = Align::new(index_ptr, mem::align_of::<u32>() as u64, index_buffer_memory_req.size);
            index_slice.copy_from_slice(&index_buffer_data);

            device.unmap_memory(index_buffer_memory);
            device.bind_buffer_memory(index_buffer, index_buffer_memory, 0).unwrap();

            let vertex_input_buffer_info = *vk::BufferCreateInfo::builder()
                .size(3 * mem::size_of::<Vertex>() as u64)
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

            let vertices = [
                Vertex {
                    pos: [-1.0, 1.0, 0.0, 1.0],
                    color: [0.0, 1.0, 0.0, 1.0],
                },
                Vertex {
                    pos: [1.0, 1.0, 0.0, 1.0],
                    color: [0.0, 0.0, 1.0, 1.0],
                },
                Vertex {
                    pos: [0.0, -1.0, 0.0, 1.0],
                    color: [1.0, 0.0, 0.0, 1.0],
                },
            ];

            let vert_ptr = device
                .map_memory(vertex_input_buffer_memory, 0, vertex_input_buffer_memory_req.size, vk::MemoryMapFlags::empty())
                .unwrap();

            let mut vert_align = Align::new(vert_ptr, mem::align_of::<Vertex>() as u64, vertex_input_buffer_memory_req.size);
            vert_align.copy_from_slice(&vertices);
            device.unmap_memory(vertex_input_buffer_memory);
            device.bind_buffer_memory(vertex_input_buffer, vertex_input_buffer_memory, 0).unwrap();
        }

        return Renderer {};
    }
}

#[derive(Clone, Debug, Copy)]
struct Vertex {
    pos: [f32; 4],
    color: [f32; 4],
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

    println!("{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n",);

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
