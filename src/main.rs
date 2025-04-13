mod clips;
mod renderer;
mod runtime;

use std::{
    env,
    io::Write,
    path::Path,
    sync::{atomic::AtomicBool, mpsc::channel, Arc},
    thread,
    time::{Duration, Instant},
};

use ash::{khr::swapchain, vk};
use cgmath::{vec2, vec4};
use clips::{ClipLoader, Clips, ScriptClip};
use deno_core::parking_lot::Mutex;
use ffmpeg_sidecar::{
    command::FfmpegCommand,
    event::{FfmpegEvent, LogLevel},
};
use log::info;

use notify::{Event, ReadDirectoryChangesWatcher, RecursiveMode, Watcher};
use renderer::{
    elements::{Elements, Rect},
    renderer::{RenderTarget, Renderer},
};
use winit::{application::ApplicationHandler, event::WindowEvent};
use winit::{dpi::LogicalSize, platform::run_on_demand::EventLoopExtRunOnDemand};
use winit::{dpi::Size, window::Window};
use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::WindowAttributes,
};

struct Editor {
    renderer: Renderer,

    swapchain: vk::SwapchainKHR,
    swapchain_loader: swapchain::Device,

    present_queue: vk::Queue,

    render_pass: vk::RenderPass,
    frame_buffers: Vec<vk::Framebuffer>,

    clip_loader: ClipLoader,
    start: Instant,
    last_frame: Instant,

    watcher: ReadDirectoryChangesWatcher,
    need_to_invalidate_paths: Arc<AtomicBool>,
    invalidated_paths: Arc<Mutex<Vec<String>>>,
}

impl Editor {
    pub fn new(window: &Window) -> Editor {
        unsafe {
            let renderer = Renderer::new_with_surface(window);

            let surface_data = renderer.surface_data.as_ref().unwrap();

            let surface_format = surface_data.surface_loader.get_physical_device_surface_formats(renderer.physical_device, surface_data.surface).unwrap()[0];

            let surface_capabilities = surface_data
                .surface_loader
                .get_physical_device_surface_capabilities(renderer.physical_device, surface_data.surface)
                .unwrap();

            let mut desired_image_count = surface_capabilities.min_image_count + 1;
            if surface_capabilities.max_image_count > 0 && desired_image_count > surface_capabilities.max_image_count {
                desired_image_count = surface_capabilities.max_image_count;
            }

            let surface_resolution = match surface_capabilities.current_extent.width {
                u32::MAX => vk::Extent2D {
                    width: window.inner_size().width,
                    height: window.inner_size().height,
                },
                _ => surface_capabilities.current_extent,
            };

            let pre_transform = if surface_capabilities.supported_transforms.contains(vk::SurfaceTransformFlagsKHR::IDENTITY) {
                vk::SurfaceTransformFlagsKHR::IDENTITY
            } else {
                surface_capabilities.current_transform
            };

            let present_modes = surface_data
                .surface_loader
                .get_physical_device_surface_present_modes(renderer.physical_device, surface_data.surface)
                .unwrap();

            let present_mode = present_modes.iter().cloned().find(|&mode| mode == vk::PresentModeKHR::MAILBOX).unwrap_or(vk::PresentModeKHR::FIFO);

            let swapchain_loader = swapchain::Device::new(&renderer.instance, &renderer.device);

            let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
                .surface(surface_data.surface)
                .min_image_count(desired_image_count)
                .image_color_space(surface_format.color_space)
                .image_format(surface_format.format)
                .image_extent(surface_resolution)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(pre_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
                .image_array_layers(1);

            let swapchain = swapchain_loader.create_swapchain(&swapchain_create_info, None).unwrap();

            let present_queue = renderer.create_graphics_queue();

            let present_images = swapchain_loader.get_swapchain_images(swapchain).unwrap();
            let present_image_views: Vec<vk::ImageView> = present_images
                .iter()
                .map(|&image| {
                    let create_view_info = vk::ImageViewCreateInfo::default()
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(surface_format.format)
                        .components(vk::ComponentMapping {
                            r: vk::ComponentSwizzle::R,
                            g: vk::ComponentSwizzle::G,
                            b: vk::ComponentSwizzle::B,
                            a: vk::ComponentSwizzle::A,
                        })
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        })
                        .image(image);
                    renderer.device.create_image_view(&create_view_info, None).unwrap()
                })
                .collect();

            let render_pass = renderer.create_render_pass(vk::ImageLayout::UNDEFINED, vk::ImageLayout::PRESENT_SRC_KHR, vk::Format::B8G8R8A8_UNORM);

            let frame_buffers: Vec<vk::Framebuffer> = present_image_views
                .iter()
                .map(|&present_image_view| {
                    let framebuffer_attachments = [present_image_view];
                    let frame_buffer_create_info = vk::FramebufferCreateInfo::default()
                        .render_pass(render_pass)
                        .attachments(&framebuffer_attachments)
                        .width(surface_resolution.width)
                        .height(surface_resolution.height)
                        .layers(1);

                    renderer.device.create_framebuffer(&frame_buffer_create_info, None).unwrap()
                })
                .collect();

            let clip_loader = ClipLoader::new();

            let invalidated_paths = Arc::new(Mutex::new(vec![]));
            let invalidated_paths_thread = invalidated_paths.clone();

            let need_to_invalidate_paths = Arc::new(AtomicBool::new(false));
            let need_to_invalidate_paths_thread = need_to_invalidate_paths.clone();

            let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| match res {
                Ok(_event) => {
                    let mut invalidated_paths = invalidated_paths_thread.lock();

                    let mut paths = _event.paths.iter().map(|path| String::from(path.to_str().unwrap()).chars().skip(28).collect()).collect::<Vec<String>>();

                    invalidated_paths.append(&mut paths);
                    need_to_invalidate_paths_thread.store(true, std::sync::atomic::Ordering::Relaxed);
                }
                _ => {}
            })
            .unwrap();

            watcher.watch(Path::new(r#"D:\Vector Engine\playground"#), RecursiveMode::Recursive).unwrap();

            return Editor {
                renderer,
                swapchain,
                swapchain_loader,
                present_queue,
                render_pass,
                frame_buffers,
                clip_loader,
                start: Instant::now(),
                last_frame: Instant::now(),
                watcher,
                invalidated_paths,
                need_to_invalidate_paths,
            };
        }
    }

    pub fn render(&mut self, window: &Window) {
        unsafe {
            let need_to_invalidate = self.need_to_invalidate_paths.swap(false, std::sync::atomic::Ordering::Relaxed);

            if need_to_invalidate {
                let mut paths = self.invalidated_paths.lock();

                for path in paths.iter() {
                    self.clip_loader.invalidate(&path);
                }

                *paths = Vec::<String>::new();
            }

            let present_finished_semaphore = self.renderer.create_semaphore();

            let (present_index, _) = self
                .swapchain_loader
                .acquire_next_image(self.swapchain, u64::MAX, present_finished_semaphore, vk::Fence::null())
                .unwrap();

            let render_target = RenderTarget::from(
                &self.renderer,
                window.inner_size().width,
                window.inner_size().height,
                self.render_pass,
                self.frame_buffers[present_index as usize],
            );

            let clip = self.clip_loader.get(&String::from("project.ts"), &self.renderer).unwrap();
            let mut clip = &mut *clip.borrow_mut();

            match clip {
                Clips::ScriptClip(script_clip) => {
                    script_clip.set_frame((self.start.elapsed().as_millis() as f64 / 1000_f64 * 60_f64) as u32 % (60 * 10));
                    script_clip.render(&self.renderer, &mut self.clip_loader, window.inner_size().width, window.inner_size().height, &render_target);
                }
                _ => {}
            }

            let wait_semaphors = [present_finished_semaphore];
            let swapchains = [self.swapchain];
            let image_indices = [present_index];
            let present_info = vk::PresentInfoKHR::default().wait_semaphores(&wait_semaphors).swapchains(&swapchains).image_indices(&image_indices);

            self.swapchain_loader.queue_present(self.present_queue, &present_info).unwrap();

            let now = Instant::now();

            let difference = self.last_frame.elapsed().as_millis() - now.elapsed().as_millis();

            if difference > 0 {
                info!("FPS {}", 1000 / difference);
            } else {
                info!("FPS too high!");
            }

            self.last_frame = now;
        }
    }
}

struct App {
    window: Option<Window>,
    editor: Option<Editor>,
}

impl App {
    pub fn new() -> App {
        return App { editor: None, window: None };
    }

    pub fn open(mut self) {
        let event_loop = EventLoop::new().unwrap();

        event_loop.run_app(&mut self).unwrap();
    }
}

impl ApplicationHandler for App {
    fn new_events(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, cause: winit::event::StartCause) {
        if self.window.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes().with_title("Vector Engine").with_inner_size(LogicalSize::new(960.0, 540.0));

        self.window = Some(event_loop.create_window(window_attributes).unwrap());

        self.editor = Some(Editor::new(self.window.as_ref().unwrap()));
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn window_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, window_id: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                if self.window.is_none() {
                    return;
                }

                match self.editor.as_mut() {
                    Some(editor) => {
                        editor.render(self.window.as_ref().unwrap());

                        self.window.as_ref().unwrap().request_redraw();
                    }
                    None => {
                        self.window.as_ref().unwrap().request_redraw();
                    }
                }
            }
            _ => {}
        }
    }
}

fn main() {
    env::set_var("RUST_LOG", "info");
    env::set_var("RUST_BACKTRACE", "1");

    pretty_env_logger::init();

    ffmpeg_sidecar::download::auto_download().unwrap();

    // let editor = App::new();
    // editor.open();

    info!("Rendering...");

    let now = Instant::now();

    let mut output = FfmpegCommand::new()
        .args(["-f", "rawvideo", "-pix_fmt", "rgba", "-s", "1920x1080", "-r", "60"])
        .input("-")
        .args(["-c:v", "libx265", "-pix_fmt", "yuva420p"])
        .args(["-y", "renders/render.mp4"])
        .spawn()
        .unwrap();

    let mut stdin = output.take_stdin().unwrap();

    let mut renderer = Renderer::new();

    let mut clip_loader = ClipLoader::new();
    let clip = clip_loader.get(&String::from("project.ts"), &renderer).unwrap();
    let mut clip = &mut *clip.borrow_mut();

    let mut total_frame_time = 0;
    let mut total_frames = 0;

    for i in 0..(60 * 8) {
        match &mut clip {
            Clips::ScriptClip(ref mut clip) => {
                clip.set_frame(i);

                let frame_now = Instant::now();
                let bytes = clip.render_to_raw(&mut renderer, &mut clip_loader, 1920, 1080);
                total_frame_time += frame_now.elapsed().as_millis();
                total_frames += 1;

                stdin.write_all(&bytes).ok();
            }
            _ => {}
        }
    }

    info!("Average frame time {}ms", total_frame_time / total_frames);

    info!("Render fully at {}ms", now.elapsed().as_millis());
}
