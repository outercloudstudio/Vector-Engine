use ash::util::read_spv;
use ash::vk::ShaderModule;
use ash::{util::Align, vk, Device, Instance};
use cgmath::{vec2, Vector2, Vector4};
use std::ffi::CStr;
use std::io::Cursor;
use std::mem::align_of;
use std::{mem, ptr::copy_nonoverlapping};

#[derive(Clone)]
pub enum Elements {
    Rect(Rect),
}

pub trait Element {
    fn render(
        &self,
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
        target_image_view: vk::ImageView,
        command_pool: vk::CommandPool,
        graphics_queue: vk::Queue,
        first_element: bool,
        last_element: bool,
    );
}

#[derive(Clone)]
pub struct Rect {
    pub position: Vector2<f32>,
    pub size: Vector2<f32>,
    pub color: Vector4<f32>,
    pub radius: f32,
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

const UVS: [Vector2<f32>; 4] = [vec2(0.0, 0.0), vec2(0.0, 1.0), vec2(1.0, 1.0), vec2(1.0, 0.0)];

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

    fn create_uniform_buffer(rect_data: RectData, instance: &Instance, device: &Device, physical_device: vk::PhysicalDevice) -> (vk::Buffer, vk::DeviceMemory) {
        unsafe {
            let uniform_buffer_object = rect_data;

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
}

impl Element for Rect {
    fn render(
        &self,
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
        target_image_view: vk::ImageView,
        command_pool: vk::CommandPool,
        graphics_queue: vk::Queue,
        first_element: bool,
        last_element: bool,
    ) {
        let command_buffer = create_command_buffer(device, command_pool);

        let render_pass = create_render_pass(device, first_element, last_element);

        let frame_buffer = create_framebuffer(device, target_image_view, render_pass);

        let vertex_shader = Rect::create_vertex_shader(device);
        let fragment_shader = Rect::create_fragment_shader(device);

        let viewport = create_viewport();
        let scissor = create_scissor();

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
        let (vertex_buffer, vertex_buffer_memory) = Rect::create_vertex_buffer(
            &vec![
                vec2(self.position.x / 1920.0, -self.position.y / 1080.0),
                vec2(self.position.x / 1920.0, -(self.position.y + self.size.y) / 1080.0),
                vec2((self.position.x + self.size.x) / 1920.0, -(self.position.y + self.size.y) / 1080.0),
                vec2((self.position.x + self.size.x) / 1920.0, -self.position.y / 1080.0),
            ],
            instance,
            device,
            physical_device,
        );
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

        begin_render_pass(device, render_pass, frame_buffer, command_buffer, graphics_pipeline, viewport, scissor);

        unsafe {
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer], &[0]);
            device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT32);
            device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline_layout, 0, &descriptor_sets, &[]);
            device.cmd_draw_indexed(command_buffer, 6, 1, 0, 0, 1);
        }

        end_render_pass(device, command_buffer, graphics_queue);

        unsafe {
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

fn create_command_buffer(device: &Device, command_pool: vk::CommandPool) -> vk::CommandBuffer {
    unsafe {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(1)
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers = device.allocate_command_buffers(&command_buffer_allocate_info).unwrap();

        command_buffers[0]
    }
}

fn create_render_pass(device: &Device, first_element: bool, last_element: bool) -> vk::RenderPass {
    unsafe {
        let color_attachment = *vk::AttachmentDescription::builder()
            .format(vk::Format::R8G8B8A8_SRGB)
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

fn create_framebuffer(device: &Device, target_image_view: vk::ImageView, render_pass: vk::RenderPass) -> vk::Framebuffer {
    unsafe {
        let frame_buffer_attachments = &[target_image_view];

        let frame_buffer_create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(frame_buffer_attachments)
            .width(480)
            .height(270)
            .layers(1);

        device.create_framebuffer(&frame_buffer_create_info, None).unwrap()
    }
}

fn create_viewport() -> vk::Viewport {
    vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: 480.0,
        height: 270.0,
        min_depth: 0.0,
        max_depth: 1.0,
    }
}

fn create_scissor() -> vk::Rect2D {
    *vk::Rect2D::builder().extent(*vk::Extent2D::builder().width(480).height(270))
}

fn create_graphics_pipeline(
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

        (graphics_pipelines[0], pipeline_layout)
    }
}

fn create_index_buffer(indices: &Vec<u32>, instance: &Instance, device: &Device, physical_device: vk::PhysicalDevice) -> (vk::Buffer, vk::DeviceMemory) {
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

fn begin_render_pass(
    device: &Device,
    render_pass: vk::RenderPass,
    frame_buffer: vk::Framebuffer,
    command_buffer: vk::CommandBuffer,
    graphics_pipeline: vk::Pipeline,
    viewport: vk::Viewport,
    scissor: vk::Rect2D,
) {
    unsafe {
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 0.0] },
        }];

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass)
            .framebuffer(frame_buffer)
            .render_area(*vk::Rect2D::builder().extent(*vk::Extent2D::builder().width(480).height(270)))
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

fn end_render_pass(device: &Device, command_buffer: vk::CommandBuffer, graphics_queue: vk::Queue) {
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
