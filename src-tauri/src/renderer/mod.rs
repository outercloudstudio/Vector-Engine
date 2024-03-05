#![allow(dead_code, unused_variables)]

use std::ffi::{c_char, CStr};
use std::ptr::copy_nonoverlapping;
use std::{borrow::Cow, default::Default};

use ash::extensions::ext::DebugUtils;
use ash::{vk, Device, Entry, Instance};

use self::elements::{Element, Elements};

type Vec2 = cgmath::Vector2<f32>;
type Vec3 = cgmath::Vector3<f32>;
type Mat4 = cgmath::Matrix4<f32>;

pub mod elements;

pub struct Renderer {
    instance: Instance,
    device: Device,
    physical_device: vk::PhysicalDevice,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
    target_image: vk::Image,
    target_image_view: vk::ImageView,
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

            let target_image_create_info = vk::ImageCreateInfo::builder()
                .image_type(vk::ImageType::TYPE_2D)
                .format(vk::Format::R8G8B8A8_UNORM)
                .extent(*vk::Extent3D::builder().width(480).height(270).depth(1))
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
                .format(vk::Format::R8G8B8A8_UNORM)
                .subresource_range(*subresource_range);

            let target_image_requirements = device.get_image_memory_requirements(target_image);

            let target_image_memory_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(target_image_requirements.size)
                .memory_type_index(get_memory_type_index(&instance, physical_device, vk::MemoryPropertyFlags::DEVICE_LOCAL, target_image_requirements));

            let target_image_memory = device.allocate_memory(&target_image_memory_info, None).unwrap();
            device.bind_image_memory(target_image, target_image_memory, 0).unwrap();

            let target_image_view = device.create_image_view(&target_image_view_create_info, None).unwrap();

            let graphics_queue = create_graphics_queue(&device, queue_family_index);

            let command_pool = create_command_pool(&device, queue_family_index);

            Renderer {
                instance,
                device,
                physical_device,
                graphics_queue,
                command_pool,
                target_image,
                target_image_view,
            }
        }
    }

    pub fn render(&mut self, elements: Vec<Elements>) -> Vec<u8> {
        for element_index in 0..elements.len() {
            let element = &elements[element_index];

            match element {
                Elements::Rect(rect) => rect.render(
                    &self.instance,
                    &self.device,
                    self.physical_device,
                    self.target_image_view,
                    self.command_pool,
                    self.graphics_queue,
                    element_index == 0,
                    element_index == elements.len() - 1,
                ),
            }
        }

        unsafe {
            let size = 480 * 270 * 4;

            let save_buffer_info = vk::BufferCreateInfo::builder()
                .size(size)
                .usage(vk::BufferUsageFlags::TRANSFER_DST)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let save_buffer = self.device.create_buffer(&save_buffer_info, None).unwrap();

            let save_buffer_memory_req = self.device.get_buffer_memory_requirements(save_buffer);
            let save_buffer_memory_index = get_memory_type_index(
                &self.instance,
                self.physical_device,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                save_buffer_memory_req,
            );

            let index_allocate_info = *vk::MemoryAllocateInfo::builder()
                .allocation_size(save_buffer_memory_req.size)
                .memory_type_index(save_buffer_memory_index);

            let save_buffer_memory = self.device.allocate_memory(&index_allocate_info, None).unwrap();

            self.device.bind_buffer_memory(save_buffer, save_buffer_memory, 0).unwrap();

            let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_pool(self.command_pool)
                .command_buffer_count(1);

            let command_buffer = self.device.allocate_command_buffers(&command_buffer_allocate_info).unwrap()[0];

            let info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            self.device.begin_command_buffer(command_buffer, &info).unwrap();

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
                .image_extent(vk::Extent3D { width: 480, height: 270, depth: 1 });

            self.device
                .cmd_copy_image_to_buffer(command_buffer, self.target_image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, save_buffer, &[region]);

            self.device.end_command_buffer(command_buffer).unwrap();

            let command_buffers = &[command_buffer];
            let info = *vk::SubmitInfo::builder().command_buffers(command_buffers);

            self.device.queue_submit(self.graphics_queue, &[info], vk::Fence::null()).unwrap();
            self.device.queue_wait_idle(self.graphics_queue).unwrap();

            self.device.free_command_buffers(self.command_pool, &[command_buffer]);

            let memory = self.device.map_memory(save_buffer_memory, 0, size, vk::MemoryMapFlags::empty()).unwrap();

            let mut pixels = vec![0; size as usize];

            copy_nonoverlapping(memory.cast(), pixels.as_mut_ptr(), size as usize);

            self.device.unmap_memory(save_buffer_memory);

            self.device.destroy_buffer(save_buffer, None);
            self.device.free_memory(save_buffer_memory, None);

            return pixels;
        }
    }
}

fn create_graphics_queue(device: &Device, queue_family_index: u32) -> vk::Queue {
    unsafe { device.get_device_queue(queue_family_index, 0) }
}

fn create_command_pool(device: &Device, queue_family_index: u32) -> vk::CommandPool {
    unsafe {
        let pool_create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family_index);

        device.create_command_pool(&pool_create_info, None).unwrap()
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
