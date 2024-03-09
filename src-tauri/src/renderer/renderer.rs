use std::ffi::{c_char, c_void, CStr};
use std::io::Cursor;
use std::ptr::copy_nonoverlapping;
use std::{borrow::Cow, default::Default};

use ash::extensions::ext::DebugUtils;
use ash::util::read_spv;
use ash::vk::ShaderModule;
use ash::{vk, Device, Entry, Instance};
use log::info;

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

    pub fn create_shader(&self, spv: Vec<u8>) -> ShaderModule {
        unsafe {
            let mut spv_file = Cursor::new(spv);

            let code = read_spv(&mut spv_file).expect("Failed to read shader spv file");
            let shader_info = vk::ShaderModuleCreateInfo::builder().code(&code);

            self.device.create_shader_module(&shader_info, None).expect("Shader module error")
        }
    }

    pub fn create_buffer(&self, size: u64, usage: vk::BufferUsageFlags, memory_property_flags: vk::MemoryPropertyFlags) -> (vk::Buffer, vk::DeviceMemory, u64) {
        unsafe {
            let buffer_info = *vk::BufferCreateInfo::builder().size(size).usage(usage).sharing_mode(vk::SharingMode::EXCLUSIVE);

            let buffer = self.device.create_buffer(&buffer_info, None).unwrap();

            let memory_requirements = self.device.get_buffer_memory_requirements(buffer);
            let memory_index = get_memory_type_index(&self.instance, self.physical_device, memory_property_flags, memory_requirements);

            let allocate_info = *vk::MemoryAllocateInfo::builder().allocation_size(memory_requirements.size).memory_type_index(memory_index);

            let memory = self.device.allocate_memory(&allocate_info, None).unwrap();

            self.device.bind_buffer_memory(buffer, memory, 0).unwrap();

            return (buffer, memory, memory_requirements.size);
        }
    }

    pub fn start_copy_data_to_buffer(&self, size: u64, memory: vk::DeviceMemory) -> *mut c_void {
        unsafe { self.device.map_memory(memory, 0, size, vk::MemoryMapFlags::empty()).unwrap() }
    }

    pub fn end_copy_data_to_buffer(&self, memory: vk::DeviceMemory) {
        unsafe {
            self.device.unmap_memory(memory);
        }
    }

    pub fn create_render_pass(&self) -> vk::RenderPass {
        unsafe {
            let color_attachment = *vk::AttachmentDescription::builder()
                .format(vk::Format::R8G8B8A8_UNORM)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::LOAD)
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

            self.device.create_render_pass(&render_pass_create_info, None).unwrap()
        }
    }

    pub fn create_framebuffer(&self, render_target: &RenderTarget, render_pass: vk::RenderPass, width: u32, height: u32) -> vk::Framebuffer {
        unsafe {
            let frame_buffer_attachments = &[render_target.target_image_view];

            let frame_buffer_create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(frame_buffer_attachments)
                .width(width)
                .height(height)
                .layers(1);

            self.device.create_framebuffer(&frame_buffer_create_info, None).unwrap()
        }
    }

    pub fn create_graphics_pipeline(
        &self,
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

            let pipeline_layout = self.device.create_pipeline_layout(&layout_create_info, None).unwrap();

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
                front_face: vk::FrontFace::CLOCKWISE,
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

            let graphics_pipelines = self
                .device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[graphic_pipeline_info.build()], None)
                .expect("Unable to create graphics pipeline");

            (graphics_pipelines[0], pipeline_layout)
        }
    }

    pub fn create_command_buffer(&self, command_pool: vk::CommandPool) -> vk::CommandBuffer {
        unsafe {
            let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_buffer_count(1)
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY);

            let command_buffers = self.device.allocate_command_buffers(&command_buffer_allocate_info).unwrap();

            command_buffers[0]
        }
    }

    pub fn begin_render_pass(
        &self,
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

            self.device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::RELEASE_RESOURCES).unwrap();

            let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            self.device.begin_command_buffer(command_buffer, &command_buffer_begin_info).expect("Begin commandbuffer");

            self.device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
            self.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline);
            self.device.cmd_set_viewport(command_buffer, 0, &[viewport]);
            self.device.cmd_set_scissor(command_buffer, 0, &[scissor]);
        }
    }

    pub fn end_render_pass(&self, command_buffer: vk::CommandBuffer, graphics_queue: vk::Queue) {
        unsafe {
            self.device.cmd_end_render_pass(command_buffer);

            self.device.end_command_buffer(command_buffer).expect("End commandbuffer");

            let command_buffers = vec![command_buffer];

            let mut submit_info = vk::SubmitInfo::builder()
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .command_buffers(&command_buffers);
            submit_info.wait_semaphore_count = 0;

            self.device.queue_submit(graphics_queue, &[submit_info.build()], vk::Fence::null()).expect("queue submit failed.");

            self.device.queue_wait_idle(graphics_queue).unwrap();
        }
    }

    pub fn create_descriptor_set_layout(&self, bindings: Vec<vk::DescriptorSetLayoutBinding>) -> vk::DescriptorSetLayout {
        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings).build();

        unsafe { self.device.create_descriptor_set_layout(&layout_info, None).unwrap() }
    }

    pub fn create_descriptor_pool(&self, pool_sizes: Vec<vk::DescriptorPoolSize>) -> vk::DescriptorPool {
        unsafe {
            let info = vk::DescriptorPoolCreateInfo::builder().pool_sizes(&pool_sizes).max_sets(1);

            self.device.create_descriptor_pool(&info, None).unwrap()
        }
    }

    pub fn create_descriptor_uniform_sets(&self, descriptor_set_layout: vk::DescriptorSetLayout, descriptor_pool: vk::DescriptorPool, uniform_buffer: vk::Buffer, size: u64) -> Vec<vk::DescriptorSet> {
        unsafe {
            let layouts = vec![descriptor_set_layout; 1];
            let info = vk::DescriptorSetAllocateInfo::builder().descriptor_pool(descriptor_pool).set_layouts(&layouts);

            let descriptor_sets = self.device.allocate_descriptor_sets(&info).unwrap();

            // Range is the size of the RectDataStruct
            let info = *vk::DescriptorBufferInfo::builder().buffer(uniform_buffer).offset(0).range(size);

            let buffer_info = &[info];
            let ubo_write = *vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[0])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(buffer_info);

            self.device.update_descriptor_sets(&[ubo_write], &[] as &[vk::CopyDescriptorSet]);

            return descriptor_sets;
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
