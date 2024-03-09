use std::ffi::{c_char, CStr};
use std::ptr::copy_nonoverlapping;
use std::{borrow::Cow, default::Default};

use ash::extensions::ext::DebugUtils;
use ash::{vk, Device, Entry, Instance};

use super::utils::*;

type Vec2 = cgmath::Vector2<f32>;
type Vec3 = cgmath::Vector3<f32>;
type Mat4 = cgmath::Matrix4<f32>;

pub struct Renderer {
    pub instance: Instance,

    pub device: Device,
    pub physical_device: vk::PhysicalDevice,

    pub queue_family_index: u32,

    pub debug_call_back: vk::DebugUtilsMessengerEXT,
    pub debug_utils: DebugUtils,
}

impl Renderer {
    pub fn new() -> Renderer {
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

            let debug_utils = DebugUtils::new(&entry, &instance);

            let debug_call_back = debug_utils.create_debug_utils_messenger(&debug_info, None).unwrap();

            let physical_devices = instance.enumerate_physical_devices().expect("Physical device error");

            let (physical_device, queue_family_index) = physical_devices
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

            let device: Device = instance.create_device(physical_device, &device_create_info, None).unwrap();

            let device_memory_properties = instance.get_physical_device_memory_properties(physical_device);

            Renderer {
                instance,
                device,
                physical_device,

                queue_family_index,

                debug_utils,
                debug_call_back,
            }
        }
    }

    pub fn destroy(self) {
        unsafe {
            self.device.destroy_device(None);

            self.debug_utils.destroy_debug_utils_messenger(self.debug_call_back, None);

            self.instance.destroy_instance(None);
        }
    }
}

pub struct RenderTarget {
    target_image: vk::Image,
    target_image_view: vk::ImageView,
    target_image_memory: vk::DeviceMemory,

    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,

    width: u32,
    height: u32,
}

impl RenderTarget {
    pub fn new(width: u32, height: u32, renderer: &Renderer) -> RenderTarget {
        unsafe {
            let target_image_create_info = vk::ImageCreateInfo::builder()
                .image_type(vk::ImageType::TYPE_2D)
                .format(vk::Format::R8G8B8A8_UNORM)
                .extent(*vk::Extent3D::builder().width(width).height(height).depth(1))
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let target_image = renderer.device.create_image(&target_image_create_info, None).unwrap();

            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);

            let target_image_view_create_info = vk::ImageViewCreateInfo::builder()
                .image(target_image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::R8G8B8A8_UNORM)
                .subresource_range(*subresource_range);

            let target_image_requirements = renderer.device.get_image_memory_requirements(target_image);

            let target_image_memory_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(target_image_requirements.size)
                .memory_type_index(get_memory_type_index(
                    &renderer.instance,
                    renderer.physical_device,
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                    target_image_requirements,
                ));

            let target_image_memory = renderer.device.allocate_memory(&target_image_memory_info, None).unwrap();
            renderer.device.bind_image_memory(target_image, target_image_memory, 0).unwrap();

            let target_image_view = renderer.device.create_image_view(&target_image_view_create_info, None).unwrap();

            let graphics_queue = create_graphics_queue(&renderer.device, renderer.queue_family_index);

            let command_pool = create_command_pool(&renderer.device, renderer.queue_family_index);

            RenderTarget {
                target_image,
                target_image_view,
                target_image_memory,

                width,
                height,

                graphics_queue,
                command_pool,
            }
        }
    }

    pub fn to_raw(&self, renderer: &Renderer) -> Vec<u8> {
        unsafe {
            let size = self.width as u64 * self.height as u64 * 4;

            let save_buffer_info = vk::BufferCreateInfo::builder()
                .size(size)
                .usage(vk::BufferUsageFlags::TRANSFER_DST)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let save_buffer = renderer.device.create_buffer(&save_buffer_info, None).unwrap();

            let save_buffer_memory_req = renderer.device.get_buffer_memory_requirements(save_buffer);
            let save_buffer_memory_index = get_memory_type_index(
                &renderer.instance,
                renderer.physical_device,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                save_buffer_memory_req,
            );

            let index_allocate_info = *vk::MemoryAllocateInfo::builder()
                .allocation_size(save_buffer_memory_req.size)
                .memory_type_index(save_buffer_memory_index);

            let save_buffer_memory = renderer.device.allocate_memory(&index_allocate_info, None).unwrap();

            renderer.device.bind_buffer_memory(save_buffer, save_buffer_memory, 0).unwrap();

            let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_pool(self.command_pool)
                .command_buffer_count(1);

            let command_buffer = renderer.device.allocate_command_buffers(&command_buffer_allocate_info).unwrap()[0];

            let info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            renderer.device.begin_command_buffer(command_buffer, &info).unwrap();

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
                .image_extent(vk::Extent3D {
                    width: self.width,
                    height: self.height,
                    depth: 1,
                });

            renderer
                .device
                .cmd_copy_image_to_buffer(command_buffer, self.target_image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, save_buffer, &[region]);

            renderer.device.end_command_buffer(command_buffer).unwrap();

            let command_buffers = &[command_buffer];
            let info = *vk::SubmitInfo::builder().command_buffers(command_buffers);

            renderer.device.queue_submit(self.graphics_queue, &[info], vk::Fence::null()).unwrap();
            renderer.device.queue_wait_idle(self.graphics_queue).unwrap();

            renderer.device.free_command_buffers(self.command_pool, &[command_buffer]);

            let memory = renderer.device.map_memory(save_buffer_memory, 0, size, vk::MemoryMapFlags::empty()).unwrap();

            let mut pixels = vec![0; size as usize];

            copy_nonoverlapping(memory.cast(), pixels.as_mut_ptr(), size as usize);

            renderer.device.unmap_memory(save_buffer_memory);

            renderer.device.destroy_buffer(save_buffer, None);
            renderer.device.free_memory(save_buffer_memory, None);

            return pixels;
        }
    }

    pub fn destroy(self, renderer: &Renderer) {
        unsafe {
            renderer.device.free_memory(self.target_image_memory, None);
            renderer.device.destroy_image_view(self.target_image_view, None);
            renderer.device.destroy_image(self.target_image, None);

            renderer.device.destroy_command_pool(self.command_pool, None);
        }
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

    println!("{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n",);

    vk::FALSE
}
