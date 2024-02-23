#![allow(dead_code, unused_variables)]

use std::collections::HashSet;

use anyhow::{anyhow, Result};
use cgmath::{point3, vec2, vec3, Deg};
use log::*;
use std::ffi::CStr;
use std::fs::File;
use std::io::BufWriter;
use std::mem::size_of;
use std::os::raw::c_void;
use std::ptr::copy_nonoverlapping as memcpy;
use thiserror::Error;
use vulkanalia::bytecode::Bytecode;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::Version;

use vulkanalia::vk::{ExtDebugUtilsExtension, MemoryRequirements};
use vulkanalia::vk::{Extent2D, MemoryTypeBuilder};

type Vec2 = cgmath::Vector2<f32>;
type Vec3 = cgmath::Vector3<f32>;
type Mat4 = cgmath::Matrix4<f32>;

pub struct Renderer {
    pub instance: Instance,
    pub device: Device,

    pub context: RenderContext,
}

struct RenderInstance {}

#[derive(Clone, Debug, Default)]
pub struct RenderContext {
    pub physical_device: vk::PhysicalDevice,

    graphics_queue: vk::Queue,

    target_image: vk::Image,
    target_image_view: vk::ImageView,
    target_image_memory: vk::DeviceMemory,

    render_pass: vk::RenderPass,
    descriptor_set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,

    framebuffer: vk::Framebuffer,

    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,

    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,

    uniform_buffer: vk::Buffer,
    uniform_buffer_memory: vk::DeviceMemory,

    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,

    texture_image: vk::Image,
    texture_image_memory: vk::DeviceMemory,
    texture_image_view: vk::ImageView,
    texture_sampler: vk::Sampler,

    messenger: vk::DebugUtilsMessengerEXT,
}

pub fn create_renderer() -> Result<Renderer> {
    unsafe {
        let mut render_context = RenderContext::default();

        let loader = LibloadingLoader::new(LIBRARY)?;
        let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;

        let instance = create_instance(&entry, &mut render_context)?;

        pick_physical_device(&instance, &mut render_context)?;

        let device = create_logical_device(&entry, &instance, &mut render_context)?;

        create_target_image(&instance, &device, &mut render_context)?;
        create_render_pass(&instance, &device, &mut render_context)?;
        create_descriptor_set_layout(&device, &mut render_context)?;
        create_pipeline(&device, &mut render_context)?;
        create_framebuffer(&device, &mut render_context)?;
        create_command_pool(&instance, &device, &mut render_context)?;

        create_texture_image(&instance, &device, &mut render_context)?;
        create_texture_image_view(&device, &mut render_context)?;
        create_texture_sampler(&device, &mut render_context)?;

        create_vertex_buffer(&instance, &device, &mut render_context)?;
        create_index_buffer(&instance, &device, &mut render_context)?;
        create_uniform_buffers(&instance, &device, &mut render_context)?;

        create_descriptor_pool(&device, &mut render_context)?;
        create_descriptor_sets(&device, &mut render_context)?;

        create_command_buffer(&device, &mut render_context)?;

        // update_uniform(0, &device, &mut render_context)?;

        // render_to_target(&device, &mut render_context)?;

        let renderer = Renderer {
            instance,
            device,
            context: render_context,
        };

        {
            renderer.instance.get_physical_device_memory_properties(renderer.context.physical_device);

            println!("[DEBUG] Got memory!");
        }

        return Ok(renderer);
    }
}

// pub fn destroy_renderer(renderer: &mut Renderer) -> Result<()> {
//     unsafe {
//         destroy_render_pipeline(&renderer)?;
//     }

//     Ok(())
// }

// pub fn render(renderer: &mut Renderer) -> Result<Vec<u8>> {
//     let mut bytes = vec![];

//     unsafe {
//         update_uniform(0, &renderer.instance.logical_device, &mut renderer.context)?;

//         render_to_target(&renderer.instance.logical_device, &mut renderer.context)?;

//         println!("Reading bytes");

//         bytes = read_target_bytes(&renderer.instance.vulkan_instance, &renderer.instance.logical_device, &mut renderer.context)?;

//         println!("Read bytes");
//     }

//     Ok(bytes)
// }

unsafe fn update_uniform(frame: u32, device: &Device, context: &mut RenderContext) -> Result<()> {
    let model = Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), Deg(90.0) * (frame as f32) / 60.0);

    let view = Mat4::look_at_rh(point3(2.0, 2.0, 2.0), point3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0));

    let mut proj = cgmath::perspective(Deg(45.0), 512_f32 / 512_f32, 0.1, 10.0);

    proj[1][1] *= -1.0;

    let ubo = UniformBufferObject { model, view, proj };

    let memory = device.map_memory(context.uniform_buffer_memory, 0, size_of::<UniformBufferObject>() as u64, vk::MemoryMapFlags::empty())?;

    memcpy(&ubo, memory.cast(), 1);

    device.unmap_memory(context.uniform_buffer_memory);

    Ok(())
}

unsafe fn render_to_target(device: &Device, context: &mut RenderContext) -> Result<()> {
    let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
    let command_buffers = &[context.command_buffers[0]];
    let submit_info = vk::SubmitInfo::builder().wait_dst_stage_mask(wait_stages).command_buffers(command_buffers).wait_semaphores(&[]);

    device.queue_submit(context.graphics_queue, &[submit_info], vk::Fence::null())?;

    device.queue_wait_idle(context.graphics_queue)?;

    Ok(())
}

unsafe fn read_target_bytes(renderer: &mut Renderer) -> Result<Vec<u8>> {
    let size = 512 * 512 * 4;

    let (buffer, buffer_memory) = create_buffer(
        &renderer.instance,
        &renderer.device,
        &renderer.context,
        size,
        vk::BufferUsageFlags::TRANSFER_DST,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let command_buffer = begin_single_time_commands(&renderer.device, &renderer.context)?;

    let subresource = vk::ImageSubresourceLayers::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .mip_level(0)
        .base_array_layer(0)
        .layer_count(1);

    let region = vk::BufferImageCopy::builder()
        .buffer_offset(0)
        .buffer_row_length(0)
        .buffer_image_height(0)
        .image_subresource(subresource)
        .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
        .image_extent(vk::Extent3D { width: 512, height: 512, depth: 1 });

    renderer
        .device
        .cmd_copy_image_to_buffer(command_buffer, renderer.context.target_image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, buffer, &[region]);

    end_single_time_commands(&renderer.device, &renderer.context, command_buffer)?;

    let memory = renderer.device.map_memory(buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

    let mut pixels = vec![0; size as usize];

    memcpy(memory.cast(), pixels.as_mut_ptr(), size as usize);

    renderer.device.unmap_memory(buffer_memory);

    renderer.device.destroy_buffer(buffer, None);
    renderer.device.free_memory(buffer_memory, None);

    Ok(pixels)
}

unsafe fn save_render(name: String, instance: &Instance, device: &Device, context: &mut RenderContext) -> Result<()> {
    let size = 512 * 512 * 4;

    let (buffer, buffer_memory) = create_buffer(
        instance,
        device,
        context,
        size,
        vk::BufferUsageFlags::TRANSFER_DST,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let command_buffer = begin_single_time_commands(device, context)?;

    let subresource = vk::ImageSubresourceLayers::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .mip_level(0)
        .base_array_layer(0)
        .layer_count(1);

    let region = vk::BufferImageCopy::builder()
        .buffer_offset(0)
        .buffer_row_length(0)
        .buffer_image_height(0)
        .image_subresource(subresource)
        .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
        .image_extent(vk::Extent3D { width: 512, height: 512, depth: 1 });

    device.cmd_copy_image_to_buffer(command_buffer, context.target_image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, buffer, &[region]);

    end_single_time_commands(device, context, command_buffer)?;

    let memory = device.map_memory(buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

    let mut pixels = vec![0; size as usize];

    memcpy(memory.cast(), pixels.as_mut_ptr(), size as usize);

    device.unmap_memory(buffer_memory);

    device.destroy_buffer(buffer, None);
    device.free_memory(buffer_memory, None);

    let file = File::create(name)?;
    let ref mut file_writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(file_writer, 512, 512);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(&pixels)?;

    Ok(())
}

// unsafe fn destroy_render_pipeline(instance: &RenderInstance, context: &RenderContext) -> Result<()> {
//     instance.logical_device.device_wait_idle().unwrap();

//     instance.logical_device.destroy_descriptor_pool(context.descriptor_pool, None);

//     instance.logical_device.destroy_buffer(context.uniform_buffer, None);
//     instance.logical_device.free_memory(context.uniform_buffer_memory, None);

//     instance.logical_device.destroy_framebuffer(context.framebuffer, None);

//     instance.logical_device.free_command_buffers(context.command_pool, &[context.command_buffers[0]]);

//     instance.logical_device.destroy_pipeline(context.pipeline, None);
//     instance.logical_device.destroy_pipeline_layout(context.pipeline_layout, None);

//     instance.logical_device.destroy_render_pass(context.render_pass, None);

//     instance.logical_device.destroy_image_view(context.target_image_view, None);
//     instance.logical_device.destroy_image(context.target_image, None);
//     instance.logical_device.free_memory(context.target_image_memory, None);

//     instance.logical_device.destroy_image_view(context.texture_image_view, None);
//     instance.logical_device.destroy_image(context.texture_image, None);
//     instance.logical_device.free_memory(context.texture_image_memory, None);

//     instance.logical_device.destroy_sampler(context.texture_sampler, None);
//     instance.logical_device.destroy_descriptor_set_layout(context.descriptor_set_layout, None);

//     instance.logical_device.destroy_buffer(context.vertex_buffer, None);
//     instance.logical_device.free_memory(context.vertex_buffer_memory, None);

//     instance.logical_device.destroy_buffer(context.index_buffer, None);
//     instance.logical_device.free_memory(context.index_buffer_memory, None);

//     instance.logical_device.destroy_command_pool(context.command_pool, None);
//     instance.logical_device.destroy_device(None);

//     if VALIDATION_ENABLED {
//         instance.vulkan_instance.destroy_debug_utils_messenger_ext(context.messenger, None);
//     }

//     instance.vulkan_instance.destroy_instance(None);

//     Ok(())
// }

const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);

const VALIDATION_ENABLED: bool = cfg!(debug_assertions);

const VALIDATION_LAYER: vk::ExtensionName = vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

unsafe fn create_instance(entry: &Entry, context: &mut RenderContext) -> Result<Instance> {
    let application_info = vk::ApplicationInfo::builder()
        .application_name(b"Vector Engine\0")
        .application_version(vk::make_version(1, 0, 0))
        .engine_name(b"No Engine\0")
        .engine_version(vk::make_version(1, 0, 0))
        .api_version(vk::make_version(1, 0, 0));

    let available_layers = entry.enumerate_instance_layer_properties()?.iter().map(|l| l.layer_name).collect::<HashSet<_>>();

    if VALIDATION_ENABLED && !available_layers.contains(&VALIDATION_LAYER) {
        return Err(anyhow!("Validation layer requested but not supported."));
    }

    let layers = if VALIDATION_ENABLED { vec![VALIDATION_LAYER.as_ptr()] } else { Vec::new() };

    let mut extensions = vec![];

    if VALIDATION_ENABLED {
        extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
    }

    // Required by Vulkan SDK on macOS since 1.3.216.
    let flags = if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION {
        info!("Enabling extensions for macOS portability.");
        extensions.push(vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_EXTENSION.name.as_ptr());
        extensions.push(vk::KHR_PORTABILITY_ENUMERATION_EXTENSION.name.as_ptr());
        vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
    } else {
        vk::InstanceCreateFlags::empty()
    };

    let mut info = vk::InstanceCreateInfo::builder()
        .application_info(&application_info)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .flags(flags);

    let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
        .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
        .user_callback(Some(debug_callback));

    if VALIDATION_ENABLED {
        info = info.push_next(&mut debug_info);
    }

    let instance = entry.create_instance(&info, None)?;

    if VALIDATION_ENABLED {
        context.messenger = instance.create_debug_utils_messenger_ext(&debug_info, None)?;
    }

    Ok(instance)
}

extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    type_: vk::DebugUtilsMessageTypeFlagsEXT,
    context: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    let context = unsafe { *context };
    let message = unsafe { CStr::from_ptr(context.message) }.to_string_lossy();

    if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
        error!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        warn!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
        debug!("({:?}) {}", type_, message);
    } else {
        trace!("({:?}) {}", type_, message);
    }

    vk::FALSE
}

#[derive(Debug, Error)]
#[error("Missing {0}.")]
pub struct SuitabilityError(pub &'static str);

unsafe fn pick_physical_device(instance: &Instance, context: &mut RenderContext) -> Result<()> {
    for physical_device in instance.enumerate_physical_devices()? {
        let properties = instance.get_physical_device_properties(physical_device);

        if let Err(error) = check_physical_device(instance, context, physical_device) {
            warn!("Skipping physical device (`{}`): {}", properties.device_name, error);
        } else {
            info!("Selected physical device (`{}`).", properties.device_name);
            context.physical_device = physical_device;
            return Ok(());
        }
    }

    Err(anyhow!("Failed to find suitable physical device."))
}

unsafe fn check_physical_device(instance: &Instance, context: &RenderContext, physical_device: vk::PhysicalDevice) -> Result<()> {
    let properties = instance.get_physical_device_properties(physical_device);
    if properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
        return Err(anyhow!(SuitabilityError("Only discrete GPUs are supported.")));
    }

    let features = instance.get_physical_device_features(physical_device);

    if features.geometry_shader != vk::TRUE {
        return Err(anyhow!(SuitabilityError("Missing geometry shader support.")));
    }

    if features.sampler_anisotropy != vk::TRUE {
        return Err(anyhow!(SuitabilityError("No sampler anisotropy.")));
    }

    QueueFamilyIndices::get(instance, context, physical_device)?;

    check_physical_device_extensions(instance, context, physical_device)?;

    Ok(())
}

const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[];

unsafe fn check_physical_device_extensions(instance: &Instance, context: &RenderContext, physical_device: vk::PhysicalDevice) -> Result<()> {
    let extensions = instance
        .enumerate_device_extension_properties(physical_device, None)?
        .iter()
        .map(|e| e.extension_name)
        .collect::<HashSet<_>>();

    if DEVICE_EXTENSIONS.iter().all(|e| extensions.contains(e)) {
        Ok(())
    } else {
        Err(anyhow!(SuitabilityError("Missing required device extensions.")))
    }
}

#[derive(Copy, Clone, Debug)]
struct QueueFamilyIndices {
    graphics: u32,
}

impl QueueFamilyIndices {
    unsafe fn get(instance: &Instance, context: &RenderContext, physical_device: vk::PhysicalDevice) -> Result<Self> {
        let properties = instance.get_physical_device_queue_family_properties(physical_device);

        let graphics = properties.iter().position(|p| p.queue_flags.contains(vk::QueueFlags::GRAPHICS)).map(|i| i as u32);

        if let Some(graphics) = graphics {
            Ok(Self { graphics })
        } else {
            Err(anyhow!(SuitabilityError("Missing required queue families.")))
        }
    }
}

unsafe fn create_logical_device(entry: &Entry, instance: &Instance, context: &mut RenderContext) -> Result<Device> {
    let indices = QueueFamilyIndices::get(instance, context, context.physical_device)?;

    let mut unique_indices = HashSet::new();
    unique_indices.insert(indices.graphics);

    let queue_priorities = &[1.0];
    let queue_infos = unique_indices
        .iter()
        .map(|i| vk::DeviceQueueCreateInfo::builder().queue_family_index(*i).queue_priorities(queue_priorities))
        .collect::<Vec<_>>();

    let layers = if VALIDATION_ENABLED { vec![VALIDATION_LAYER.as_ptr()] } else { vec![] };

    let mut extensions = DEVICE_EXTENSIONS.iter().map(|n| n.as_ptr()).collect::<Vec<_>>();

    // Required by Vulkan SDK on macOS since 1.3.216.
    if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION {
        extensions.push(vk::KHR_PORTABILITY_SUBSET_EXTENSION.name.as_ptr());
    }

    let features = vk::PhysicalDeviceFeatures::builder().sampler_anisotropy(true);

    let info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .enabled_features(&features);

    let device = instance.create_device(context.physical_device, &info, None)?;

    context.graphics_queue = device.get_device_queue(indices.graphics, 0);

    Ok(device)
}

unsafe fn create_target_image(instance: &Instance, device: &Device, context: &mut RenderContext) -> Result<()> {
    let info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::_2D)
        .extent(vk::Extent3D { width: 512, height: 512, depth: 1 })
        .mip_levels(1)
        .array_layers(1)
        .format(vk::Format::R8G8B8A8_SRGB)
        .tiling(vk::ImageTiling::OPTIMAL)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC)
        .samples(vk::SampleCountFlags::_1)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let image = device.create_image(&info, None)?;

    let requirements = device.get_image_memory_requirements(image);

    let info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(instance, context, vk::MemoryPropertyFlags::DEVICE_LOCAL, requirements)?);

    let image_memory = device.allocate_memory(&info, None)?;

    device.bind_image_memory(image, image_memory, 0)?;

    context.target_image = image;
    context.target_image_memory = image_memory;

    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1);

    let info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::_2D)
        .format(vk::Format::R8G8B8A8_SRGB)
        .subresource_range(subresource_range);

    context.target_image_view = device.create_image_view(&info, None)?;

    Ok(())
}

unsafe fn get_memory_type_index(instance: &Instance, context: &RenderContext, properties: vk::MemoryPropertyFlags, requirements: vk::MemoryRequirements) -> Result<u32> {
    let memory = instance.get_physical_device_memory_properties(context.physical_device);

    (0..memory.memory_type_count)
        .find(|i| {
            let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
            let memory_type = memory.memory_types[*i as usize];

            suitable && memory_type.property_flags.contains(properties)
        })
        .ok_or_else(|| anyhow!("Failed to find suitable memory type."))
}

unsafe fn create_pipeline(device: &Device, context: &mut RenderContext) -> Result<()> {
    let vert = include_bytes!("../shaders/vert.spv");
    let frag = include_bytes!("../shaders/frag.spv");

    let vert_shader_module = create_shader_module(device, &vert[..])?;
    let frag_shader_module = create_shader_module(device, &frag[..])?;

    let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_shader_module)
        .name(b"main\0");

    let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_shader_module)
        .name(b"main\0");

    let binding_descriptions = &[Vertex::binding_description()];
    let attribute_descriptions = Vertex::attribute_descriptions();
    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(binding_descriptions)
        .vertex_attribute_descriptions(&attribute_descriptions);

    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);

    let viewport = vk::Viewport::builder().x(0.0).y(0.0).width(512_f32).height(512_f32).min_depth(0.0).max_depth(1.0);

    let scissor = vk::Rect2D::builder().offset(vk::Offset2D { x: 0, y: 0 }).extent(Extent2D { width: 512, height: 512 });

    let viewports = &[viewport];
    let scissors = &[scissor];
    let viewport_state = vk::PipelineViewportStateCreateInfo::builder().viewports(viewports).scissors(scissors);

    let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE) // Projection flip requires us to reverse this
        .depth_bias_enable(false);

    let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(false)
        .rasterization_samples(vk::SampleCountFlags::_1);

    let attachment = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::all())
        .blend_enable(false)
        .src_color_blend_factor(vk::BlendFactor::ONE) // Optional
        .dst_color_blend_factor(vk::BlendFactor::ZERO) // Optional
        .color_blend_op(vk::BlendOp::ADD) // Optional
        .src_alpha_blend_factor(vk::BlendFactor::ONE) // Optional
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO) // Optional
        .alpha_blend_op(vk::BlendOp::ADD); // Optional

    let attachments = &[attachment];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0]);

    let set_layouts = &[context.descriptor_set_layout];
    let layout_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(set_layouts);

    context.pipeline_layout = device.create_pipeline_layout(&layout_info, None)?;

    let stages = &[vert_stage, frag_stage];
    let info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(stages)
        .vertex_input_state(&vertex_input_state)
        .input_assembly_state(&input_assembly_state)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .multisample_state(&multisample_state)
        .color_blend_state(&color_blend_state)
        .layout(context.pipeline_layout)
        .render_pass(context.render_pass)
        .subpass(0)
        .base_pipeline_handle(vk::Pipeline::null())
        .base_pipeline_index(-1);

    context.pipeline = device.create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)?.0[0];

    device.destroy_shader_module(vert_shader_module, None);
    device.destroy_shader_module(frag_shader_module, None);

    Ok(())
}

unsafe fn create_shader_module(device: &Device, bytecode: &[u8]) -> Result<vk::ShaderModule> {
    let bytecode = Bytecode::new(bytecode).unwrap();

    let info = vk::ShaderModuleCreateInfo::builder().code_size(bytecode.code_size()).code(bytecode.code());

    Ok(device.create_shader_module(&info, None)?)
}

unsafe fn create_render_pass(instance: &Instance, device: &Device, context: &mut RenderContext) -> Result<()> {
    let color_attachment = vk::AttachmentDescription::builder()
        .format(vk::Format::R8G8B8A8_SRGB)
        .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL);

    let color_attachment_ref = vk::AttachmentReference::builder().attachment(0).layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let color_attachments = &[color_attachment_ref];
    let subpass = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(color_attachments);

    let dependency = vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

    let attachments = &[color_attachment];
    let subpasses = &[subpass];
    let dependencies = &[dependency];
    let info = vk::RenderPassCreateInfo::builder().attachments(attachments).subpasses(subpasses).dependencies(dependencies);

    context.render_pass = device.create_render_pass(&info, None)?;

    Ok(())
}

unsafe fn create_framebuffer(device: &Device, context: &mut RenderContext) -> Result<()> {
    let attachments = &[context.target_image_view];

    let create_info = vk::FramebufferCreateInfo::builder()
        .render_pass(context.render_pass)
        .attachments(attachments)
        .width(512)
        .height(512)
        .layers(1);

    context.framebuffer = device.create_framebuffer(&create_info, None)?;

    Ok(())
}

unsafe fn create_command_pool(instance: &Instance, device: &Device, context: &mut RenderContext) -> Result<()> {
    let indices = QueueFamilyIndices::get(instance, context, context.physical_device)?;

    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::empty()) // Optional.
        .queue_family_index(indices.graphics);

    context.command_pool = device.create_command_pool(&info, None)?;

    Ok(())
}

unsafe fn create_command_buffer(device: &Device, context: &mut RenderContext) -> Result<()> {
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(context.command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);

    context.command_buffers = device.allocate_command_buffers(&allocate_info)?;

    let inheritance = vk::CommandBufferInheritanceInfo::builder();

    let info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::empty()) // Optional.
        .inheritance_info(&inheritance); // Optional.

    device.begin_command_buffer(context.command_buffers[0], &info)?;

    let render_area = vk::Rect2D::builder().offset(vk::Offset2D::default()).extent(Extent2D { width: 512, height: 512 });

    let color_clear_value = vk::ClearValue {
        color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 1.0] },
    };

    let clear_values = &[color_clear_value];
    let info = vk::RenderPassBeginInfo::builder()
        .render_pass(context.render_pass)
        .framebuffer(context.framebuffer)
        .render_area(render_area)
        .clear_values(clear_values);

    device.cmd_begin_render_pass(context.command_buffers[0], &info, vk::SubpassContents::INLINE);

    device.cmd_bind_pipeline(context.command_buffers[0], vk::PipelineBindPoint::GRAPHICS, context.pipeline);

    device.cmd_bind_vertex_buffers(context.command_buffers[0], 0, &[context.vertex_buffer], &[0]);
    device.cmd_bind_index_buffer(context.command_buffers[0], context.index_buffer, 0, vk::IndexType::UINT16);

    device.cmd_bind_descriptor_sets(
        context.command_buffers[0],
        vk::PipelineBindPoint::GRAPHICS,
        context.pipeline_layout,
        0,
        &[context.descriptor_sets[0]],
        &[],
    );

    device.cmd_draw_indexed(context.command_buffers[0], INDICES.len() as u32, 1, 0, 0, 0);

    device.cmd_end_render_pass(context.command_buffers[0]);

    device.end_command_buffer(context.command_buffers[0])?;

    Ok(())
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    pos: Vec2,
    color: Vec3,
    tex_coord: Vec2,
}

impl Vertex {
    const fn new(pos: Vec2, color: Vec3, tex_coord: Vec2) -> Self {
        Self { pos, color, tex_coord }
    }

    fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
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
            .offset(size_of::<Vec2>() as u32)
            .build();
        let tex_coord = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(2)
            .format(vk::Format::R32G32_SFLOAT)
            .offset((size_of::<Vec2>() + size_of::<Vec3>()) as u32)
            .build();
        [pos, color, tex_coord]
    }
}

static VERTICES: [Vertex; 4] = [
    Vertex::new(vec2(-0.5, -0.5), vec3(1.0, 0.0, 0.0), vec2(1.0, 0.0)),
    Vertex::new(vec2(0.5, -0.5), vec3(0.0, 1.0, 0.0), vec2(0.0, 0.0)),
    Vertex::new(vec2(0.5, 0.5), vec3(0.0, 0.0, 1.0), vec2(0.0, 1.0)),
    Vertex::new(vec2(-0.5, 0.5), vec3(1.0, 1.0, 1.0), vec2(1.0, 1.0)),
];

const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

unsafe fn create_vertex_buffer(instance: &Instance, device: &Device, context: &mut RenderContext) -> Result<()> {
    let size = (size_of::<Vertex>() * VERTICES.len()) as u64;

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance,
        device,
        context,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

    memcpy(VERTICES.as_ptr(), memory.cast(), VERTICES.len());

    device.unmap_memory(staging_buffer_memory);

    let (vertex_buffer, vertex_buffer_memory) = create_buffer(
        instance,
        device,
        context,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    context.vertex_buffer = vertex_buffer;
    context.vertex_buffer_memory = vertex_buffer_memory;

    copy_buffer(device, context, staging_buffer, vertex_buffer, size)?;

    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok(())
}

unsafe fn create_index_buffer(instance: &Instance, device: &Device, context: &mut RenderContext) -> Result<()> {
    let size = (size_of::<u16>() * INDICES.len()) as u64;

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance,
        device,
        context,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

    memcpy(INDICES.as_ptr(), memory.cast(), INDICES.len());

    device.unmap_memory(staging_buffer_memory);

    let (index_buffer, index_buffer_memory) = create_buffer(
        instance,
        device,
        context,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    context.index_buffer = index_buffer;
    context.index_buffer_memory = index_buffer_memory;

    copy_buffer(device, context, staging_buffer, index_buffer, size)?;

    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok(())
}

unsafe fn create_buffer(
    instance: &Instance,
    device: &Device,
    context: &RenderContext,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let buffer_info = vk::BufferCreateInfo::builder().size(size).usage(usage).sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = device.create_buffer(&buffer_info, None)?;

    let requirements = device.get_buffer_memory_requirements(buffer);

    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(instance, context, properties, requirements)?);

    let buffer_memory = device.allocate_memory(&memory_info, None)?;

    device.bind_buffer_memory(buffer, buffer_memory, 0)?;

    Ok((buffer, buffer_memory))
}

unsafe fn copy_buffer(device: &Device, context: &RenderContext, source: vk::Buffer, destination: vk::Buffer, size: vk::DeviceSize) -> Result<()> {
    let command_buffer = begin_single_time_commands(device, context)?;

    let regions = vk::BufferCopy::builder().size(size);
    device.cmd_copy_buffer(command_buffer, source, destination, &[regions]);

    end_single_time_commands(device, context, command_buffer)?;

    Ok(())
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct UniformBufferObject {
    model: Mat4,
    view: Mat4,
    proj: Mat4,
}

unsafe fn create_descriptor_set_layout(device: &Device, context: &mut RenderContext) -> Result<()> {
    let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX);

    let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);

    let bindings = &[ubo_binding, sampler_binding];
    let info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings);

    context.descriptor_set_layout = device.create_descriptor_set_layout(&info, None)?;

    Ok(())
}

unsafe fn create_uniform_buffers(instance: &Instance, device: &Device, context: &mut RenderContext) -> Result<()> {
    let (uniform_buffer, uniform_buffer_memory) = create_buffer(
        instance,
        device,
        context,
        size_of::<UniformBufferObject>() as u64,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    context.uniform_buffer = uniform_buffer;
    context.uniform_buffer_memory = uniform_buffer_memory;

    Ok(())
}

unsafe fn create_descriptor_pool(device: &Device, context: &mut RenderContext) -> Result<()> {
    let ubo_size = vk::DescriptorPoolSize::builder().type_(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(1);

    let sampler_size = vk::DescriptorPoolSize::builder().type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1);

    let pool_sizes = &[ubo_size, sampler_size];
    let info = vk::DescriptorPoolCreateInfo::builder().pool_sizes(pool_sizes).max_sets(1);

    context.descriptor_pool = device.create_descriptor_pool(&info, None)?;

    Ok(())
}

unsafe fn create_descriptor_sets(device: &Device, context: &mut RenderContext) -> Result<()> {
    let layouts = vec![context.descriptor_set_layout; 1];
    let info = vk::DescriptorSetAllocateInfo::builder().descriptor_pool(context.descriptor_pool).set_layouts(&layouts);

    context.descriptor_sets = device.allocate_descriptor_sets(&info)?;

    let info = vk::DescriptorBufferInfo::builder()
        .buffer(context.uniform_buffer)
        .offset(0)
        .range(size_of::<UniformBufferObject>() as u64);

    let buffer_info = &[info];
    let ubo_write = vk::WriteDescriptorSet::builder()
        .dst_set(context.descriptor_sets[0])
        .dst_binding(0)
        .dst_array_element(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .buffer_info(buffer_info);

    let info = vk::DescriptorImageInfo::builder()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(context.texture_image_view)
        .sampler(context.texture_sampler);

    let image_info = &[info];
    let sampler_write = vk::WriteDescriptorSet::builder()
        .dst_set(context.descriptor_sets[0])
        .dst_binding(1)
        .dst_array_element(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .image_info(image_info);

    device.update_descriptor_sets(&[ubo_write, sampler_write], &[] as &[vk::CopyDescriptorSet]);

    Ok(())
}

unsafe fn begin_single_time_commands(device: &Device, context: &RenderContext) -> Result<vk::CommandBuffer> {
    let info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(context.command_pool)
        .command_buffer_count(1);

    let command_buffer = device.allocate_command_buffers(&info)?[0];

    let info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    device.begin_command_buffer(command_buffer, &info)?;

    Ok(command_buffer)
}

unsafe fn end_single_time_commands(device: &Device, context: &RenderContext, command_buffer: vk::CommandBuffer) -> Result<()> {
    device.end_command_buffer(command_buffer)?;

    let command_buffers = &[command_buffer];
    let info = vk::SubmitInfo::builder().command_buffers(command_buffers);

    device.queue_submit(context.graphics_queue, &[info], vk::Fence::null())?;
    device.queue_wait_idle(context.graphics_queue)?;

    device.free_command_buffers(context.command_pool, &[command_buffer]);

    Ok(())
}

unsafe fn create_texture_sampler(device: &Device, context: &mut RenderContext) -> Result<()> {
    let info = vk::SamplerCreateInfo::builder()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::REPEAT)
        .address_mode_v(vk::SamplerAddressMode::REPEAT)
        .address_mode_w(vk::SamplerAddressMode::REPEAT)
        .anisotropy_enable(true)
        .max_anisotropy(16.0)
        .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
        .unnormalized_coordinates(false)
        .compare_enable(false)
        .compare_op(vk::CompareOp::ALWAYS)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .mip_lod_bias(0.0)
        .min_lod(0.0)
        .max_lod(0.0);

    context.texture_sampler = device.create_sampler(&info, None)?;

    Ok(())
}

unsafe fn create_texture_image_view(device: &Device, context: &mut RenderContext) -> Result<()> {
    context.texture_image_view = create_image_view(device, context.texture_image, vk::Format::R8G8B8A8_SRGB)?;

    Ok(())
}

unsafe fn create_image_view(device: &Device, image: vk::Image, format: vk::Format) -> Result<vk::ImageView> {
    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1);

    let info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::_2D)
        .format(format)
        .subresource_range(subresource_range);

    Ok(device.create_image_view(&info, None)?)
}

unsafe fn create_texture_image(instance: &Instance, device: &Device, context: &mut RenderContext) -> Result<()> {
    // Load the image bytes
    let image = File::open("resources/texture.png")?;

    let decoder = png::Decoder::new(image);
    let mut reader = decoder.read_info()?;

    let mut pixels = vec![0; reader.info().raw_bytes()];
    reader.next_frame(&mut pixels)?;

    let size = reader.info().raw_bytes() as u64;
    let (width, height) = reader.info().size();

    // Create a buffer to load image bytes one, we'll use this to move the bytes onto a device local image
    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance,
        device,
        context,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

    memcpy(pixels.as_ptr(), memory.cast(), pixels.len());

    device.unmap_memory(staging_buffer_memory);

    // Create an image and memory to save locally on the GPU
    let (texture_image, texture_image_memory) = create_image(
        instance,
        device,
        context,
        width,
        height,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    context.texture_image = texture_image;
    context.texture_image_memory = texture_image_memory;

    // Copy buffer context to image
    transition_image_layout(
        device,
        context,
        context.texture_image,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
    )?;

    copy_buffer_to_image(device, context, staging_buffer, context.texture_image, width, height)?;

    transition_image_layout(
        device,
        context,
        context.texture_image,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    )?;

    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok(())
}

unsafe fn create_image(
    instance: &Instance,
    device: &Device,
    context: &RenderContext,
    width: u32,
    height: u32,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Image, vk::DeviceMemory)> {
    let info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::_2D)
        .extent(vk::Extent3D { width, height, depth: 1 })
        .mip_levels(1)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .samples(vk::SampleCountFlags::_1)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let image = device.create_image(&info, None)?;

    let requirements = device.get_image_memory_requirements(image);

    let info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(instance, context, properties, requirements)?);

    let image_memory = device.allocate_memory(&info, None)?;

    device.bind_image_memory(image, image_memory, 0)?;

    Ok((image, image_memory))
}

unsafe fn transition_image_layout(device: &Device, context: &RenderContext, image: vk::Image, format: vk::Format, old_layout: vk::ImageLayout, new_layout: vk::ImageLayout) -> Result<()> {
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
        _ => return Err(anyhow!("Unsupported image layout transition!")),
    };

    let command_buffer = begin_single_time_commands(device, context)?;

    let subresource = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1);

    let barrier = vk::ImageMemoryBarrier::builder()
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

    end_single_time_commands(device, context, command_buffer)?;

    Ok(())
}

unsafe fn copy_buffer_to_image(device: &Device, context: &RenderContext, buffer: vk::Buffer, image: vk::Image, width: u32, height: u32) -> Result<()> {
    let command_buffer = begin_single_time_commands(device, context)?;

    let subresource = vk::ImageSubresourceLayers::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .mip_level(0)
        .base_array_layer(0)
        .layer_count(1);

    let region = vk::BufferImageCopy::builder()
        .buffer_offset(0)
        .buffer_row_length(0)
        .buffer_image_height(0)
        .image_subresource(subresource)
        .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
        .image_extent(vk::Extent3D { width, height, depth: 1 });

    device.cmd_copy_buffer_to_image(command_buffer, buffer, image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[region]);

    end_single_time_commands(device, context, command_buffer)?;

    Ok(())
}
