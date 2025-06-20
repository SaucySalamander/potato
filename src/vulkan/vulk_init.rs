use super::command_pool::{create_command_buffers, create_command_pool};
use super::constants::{INDICES_DATA, MAX_FRAMES_IN_FLIGHT, VALIDATION, VERTICES_DATA};
use super::device::create_logical_device;
use super::framebuffers::create_framebuffers;
use super::graphics_pipeline::create_graphics_pipeline;
use super::instance::create_instance;
use super::physical_device::{describe_device, select_physical_device};
use super::queue_family::QueueFamily;
use super::render_pass::create_render_pass;
use super::surface::{create_surface, PotatoSurface};
use super::swapchain::{create_swapchain, PotatoSwapChain};
use super::sync_objects::create_sync_objects;
use super::vertex::{create_index_buffer, create_vertex_buffer};
use super::vulk_validation_layers::setup_debug_utils;
use super::UniformBufferObject::{
    create_descriptor_pool, create_descriptor_set_layout, create_descriptor_sets,
    create_uniform_buffers, update_uniform_buffer,
};
use ash::extensions::ext::DebugUtils;
use ash::vk::{
    Buffer, BufferUsageFlags, CommandBuffer, CommandPool, DebugUtilsMessengerEXT, DescriptorPool,
    DescriptorSet, DescriptorSetLayout, DeviceMemory, Fence, Framebuffer, PhysicalDevice, Pipeline,
    PipelineLayout, PipelineStageFlags, PresentInfoKHR, Queue, RenderPass, Result, Semaphore,
    StructureType, SubmitInfo,
};
use ash::{Device, Entry, Instance};
use log::debug;
use std::collections::HashMap;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder, WindowId},
};

pub struct VulkanApiObjects {
    windows: Option<HashMap<WindowId, Window>>,
    entry: Entry,
    instance: Instance,
    surface: Option<PotatoSurface>,
    queue_family: QueueFamily,
    debug_utils_loader: Option<DebugUtils>,
    debug_messenger: Option<DebugUtilsMessengerEXT>,
    physical_device: PhysicalDevice,
    device: Device,
    graphics_queue: Queue,
    swapchain: Option<PotatoSwapChain>,
    pipeline_layout: PipelineLayout,
    render_pass: RenderPass,
    graphics_pipeline: Pipeline,
    swapchain_framebuffers: Option<Vec<Framebuffer>>,
    command_pool: CommandPool,
    command_buffers: Vec<CommandBuffer>,
    image_available_semaphores: Option<Vec<Semaphore>>,
    render_finished_semaphores: Option<Vec<Semaphore>>,
    in_flight_fences: Vec<Fence>,
    current_frame: usize,
    vertex_buffer: Buffer,
    vertex_buffer_memory: DeviceMemory,
    index_buffer: Buffer,
    index_buffer_memory: DeviceMemory,
    uniform_buffers: Vec<Buffer>,
    uniform_buffers_memory: Vec<DeviceMemory>,
    ubo_layout: DescriptorSetLayout,
    descriptor_pool: DescriptorPool,
    descriptor_sets: Vec<DescriptorSet>,
}

impl VulkanApiObjects {
    //TODO Does not currently work in the lib and as referenced outside the lib
    pub fn init(event_loop: &EventLoop<()>) -> VulkanApiObjects {
        debug!("Init window");
        let window = VulkanApiObjects::init_window(&event_loop, "origin");
        debug!("Init entry");
        let entry = Entry::linked();
        debug!("Init instance");
        let instance = create_instance(&entry);
        debug!("Init debug utils");
        let (debug_utils_loader, debug_messenger) = setup_debug_utils(&entry, &instance);
        debug!("Init surface");
        let potato_surface = create_surface(&entry, &instance, &window);
        debug!("Init physical device");
        let physical_device = select_physical_device(&instance, &potato_surface);
        let physical_device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };
        describe_device(&instance, physical_device);

        debug!("Init logical device");
        let (logical_device, queue_family) =
            create_logical_device(&instance, physical_device, &potato_surface);
        debug!("Init swapchain");
        let swapchain = create_swapchain(
            &instance,
            &logical_device,
            physical_device,
            &potato_surface,
            &queue_family,
        );
        debug!("Init graphics queue");
        let graphics_queue = unsafe {
            logical_device.get_device_queue(queue_family.graphics_family.unwrap() as u32, 0)
        };
        debug!("Init render pass");
        let render_pass = create_render_pass(&logical_device, swapchain.swapchain_format);
        debug!("Init descriptor layout");
        let ubo_layout = create_descriptor_set_layout(&logical_device);
        debug!("Init graphics pipeline");
        let (graphics_pipeline, pipeline_layout) = create_graphics_pipeline(
            &logical_device,
            render_pass,
            swapchain.swapchain_extent,
            ubo_layout,
        );
        debug!("Init framebuffers");
        let swapchain_framebuffers = create_framebuffers(
            &logical_device,
            render_pass,
            &swapchain.swapchain_image_views,
            &swapchain.swapchain_extent,
        );
        debug!("Init command pool");
        let command_pool = create_command_pool(&logical_device, &queue_family);
        debug!("Init vertex buffer");
        let (vertex_buffer, vertex_buffer_memory) = create_vertex_buffer(
            &instance,
            &logical_device,
            physical_device,
            command_pool,
            graphics_queue,
            BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::VERTEX_BUFFER,
        );
        debug!("Init index buffer");
        let (index_buffer, index_buffer_memory) = create_index_buffer(
            &instance,
            &logical_device,
            physical_device,
            command_pool,
            graphics_queue,
            BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::INDEX_BUFFER,
        );
        debug!("Init ubo buffer");
        let (uniform_buffers, uniform_buffers_memory) = create_uniform_buffers(
            &logical_device,
            &physical_device_memory_properties,
            swapchain.swapchain_images.len(),
        );
        debug!("Init descriptor pool");
        let descriptor_pool =
            create_descriptor_pool(&logical_device, swapchain.swapchain_images.len());
        debug!("Init descriptor sets");
        let descriptor_sets = create_descriptor_sets(
            &logical_device,
            descriptor_pool,
            ubo_layout,
            &uniform_buffers,
            swapchain.swapchain_images.len(),
        );
        debug!("Init command buffers");
        let command_buffers = create_command_buffers(
            &logical_device,
            command_pool,
            graphics_pipeline,
            &swapchain_framebuffers,
            render_pass,
            swapchain.swapchain_extent,
            vertex_buffer,
            index_buffer,
            pipeline_layout,
            &descriptor_sets,
        );
        debug!("Init sync objects");
        let sync_objects = create_sync_objects(&logical_device);

        let mut windows = HashMap::new();
        windows.insert(window.id(), window);

        VulkanApiObjects {
            windows: Some(windows),
            entry: entry,
            instance,
            surface: Some(potato_surface),
            queue_family,
            debug_utils_loader: Some(debug_utils_loader),
            debug_messenger: Some(debug_messenger),
            physical_device,
            device: logical_device,
            graphics_queue,
            swapchain: Some(swapchain),
            pipeline_layout,
            render_pass,
            graphics_pipeline,
            swapchain_framebuffers: Some(swapchain_framebuffers),
            command_pool,
            command_buffers,
            image_available_semaphores: Some(sync_objects.image_available_semaphores),
            render_finished_semaphores: Some(sync_objects.render_finished_semaphores),
            in_flight_fences: sync_objects.inflight_fences,
            current_frame: 0,
            vertex_buffer,
            vertex_buffer_memory,
            index_buffer,
            index_buffer_memory,
            uniform_buffers,
            uniform_buffers_memory,
            ubo_layout,
            descriptor_pool,
            descriptor_sets,
        }
    }

    pub fn init_compute() -> VulkanApiObjects {
        debug!("Linking entry");
        let entry = Entry::linked();
        debug!("Creating instance");
        let instance = create_instance(&entry);

        VulkanApiObjects {
            windows: None,
            entry: entry,
            instance: instance,
            surface: None,
            queue_family: None,
            debug_utils_loader: None,
            debug_messenger: None,
            physical_device: None,
            device: None,
            graphics_queue: None,
            swapchain: None,
            pipeline_layout: None,
            render_pass: None,
            graphics_pipeline: None,
            swapchain_framebuffers: None,
            command_pool: None,
            command_buffers: None,
            image_available_semaphores: None,
            render_finished_semaphores: None,
            in_flight_fences: None,
            current_frame: None,
            vertex_buffer: None,
            vertex_buffer_memory: None,
            index_buffer: None,
            index_buffer_memory: None,
            uniform_buffers: None,
            uniform_buffers_memory: None,
            ubo_layout: None,
            descriptor_pool: None,
            descriptor_sets: None,
        }
    }

    pub fn draw(&mut self, delta_time: f32) {
        let wait_fences = [self.in_flight_fences[self.current_frame]];
        let (image_index, _is_sub_optimal) = unsafe {
            self.device
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");

            let result = self.swapchain.swapchain_loader.acquire_next_image(
                self.swapchain.swapchain,
                std::u64::MAX,
                self.image_available_semaphores[self.current_frame],
                Fence::null(),
            );
            match result {
                Ok(image_index) => image_index,
                Err(vk_result) => match vk_result {
                    Result::ERROR_OUT_OF_DATE_KHR => {
                        self.recreate_swapchain();
                        return;
                    }
                    _ => panic!("Failed to acquire swap chain image"),
                },
            }
        };

        update_uniform_buffer(
            &self.swapchain,
            &self.device,
            image_index as usize,
            delta_time,
            &self.uniform_buffers_memory,
        );

        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let wait_stages = [PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];

        let submit_infos = [SubmitInfo {
            s_type: StructureType::SUBMIT_INFO,
            p_next: std::ptr::null(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &self.command_buffers[image_index as usize],
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
        }];
        unsafe {
            self.device
                .reset_fences(&wait_fences)
                .expect("Failed to reset Fence!");

            self.device
                .queue_submit(
                    self.graphics_queue,
                    &submit_infos,
                    self.in_flight_fences[self.current_frame],
                )
                .expect("Failed to execute queue submit.");
        }

        let swapchains = [self.swapchain.swapchain];

        let present_info = PresentInfoKHR {
            s_type: StructureType::PRESENT_INFO_KHR,
            p_next: std::ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &image_index,
            p_results: std::ptr::null_mut(),
        };

        let result = unsafe {
            self.swapchain
                .swapchain_loader
                .queue_present(self.graphics_queue, &present_info)
        };

        let is_resized = match result {
            Ok(_) => false,
            Err(vk_result) => match vk_result {
                Result::ERROR_OUT_OF_DATE_KHR | Result::SUBOPTIMAL_KHR => true,
                _ => panic!("Failed to execute queue present"),
            },
        };

        if is_resized {
            self.recreate_swapchain();
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    fn recreate_swapchain(&mut self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("Failed to wait on device")
        };
        self.cleanup_swapchain();

        self.swapchain = create_swapchain(
            &self.instance,
            &self.device,
            self.physical_device,
            &self.surface,
            &self.queue_family,
        );
        self.render_pass = create_render_pass(&self.device, self.swapchain.swapchain_format);
        let (graphics_pipeline, pipeline_layout) = create_graphics_pipeline(
            &self.device,
            self.render_pass,
            self.swapchain.swapchain_extent,
            self.ubo_layout,
        );
        self.graphics_pipeline = graphics_pipeline;
        self.pipeline_layout = pipeline_layout;
        self.swapchain_framebuffers = create_framebuffers(
            &self.device,
            self.render_pass,
            &self.swapchain.swapchain_image_views,
            &self.swapchain.swapchain_extent,
        );
        self.command_buffers = create_command_buffers(
            &self.device,
            self.command_pool,
            graphics_pipeline,
            &self.swapchain_framebuffers,
            self.render_pass,
            self.swapchain.swapchain_extent,
            self.vertex_buffer,
            self.index_buffer,
            self.pipeline_layout,
            &self.descriptor_sets,
        );
    }

    fn cleanup_swapchain(&self) {
        unsafe {
            self.device
                .free_command_buffers(self.command_pool, &self.command_buffers);
            self.swapchain_framebuffers
                .iter()
                .for_each(|x| self.device.destroy_framebuffer(*x, None));
            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
            self.swapchain
                .swapchain_image_views
                .iter()
                .for_each(|x| self.device.destroy_image_view(*x, None));
            self.swapchain
                .swapchain_loader
                .destroy_swapchain(self.swapchain.swapchain, None);
        }
    }

    fn init_window(event_loop: &EventLoopWindowTarget<()>, name: &str) -> Window {
        WindowBuilder::new()
            .with_title(name)
            .with_inner_size(LogicalSize::new(800, 600))
            .build(event_loop)
            .expect("Failed to create window.")
    }

    pub fn init_event_loop(mut self, event_loop: EventLoop<()>) {
        let time = std::time::Instant::now();
        let mut delta_frame = 0;
        event_loop.run(move |event, event_loop, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent { event, window_id } => {
                    if let WindowEvent::CloseRequested = event {
                        println!("Window {:?} has received the signal to close", window_id);
                        self.windows.remove(&window_id);
                        if self.windows.is_empty() {
                            *control_flow = ControlFlow::Exit;
                        }
                    }

                    if let WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode,
                                state,
                                ..
                            },
                        is_synthetic,
                        ..
                    } = event
                    {
                        //TODO abstract keyboard input logic
                        if state == ElementState::Released
                            && virtual_keycode == Some(VirtualKeyCode::N)
                            && !is_synthetic
                        {
                            let window = VulkanApiObjects::init_window(event_loop, "spawn");
                            self.windows.insert(window.id(), window);
                        }
                    }
                }
                Event::MainEventsCleared => {
                    for (.., window) in self.windows.iter() {
                        window.request_redraw();
                    }
                }
                Event::RedrawRequested(_window_id) => {
                    let delta_time = delta_frame as f32 / 1_000_000.0 as f32;
                    self.draw(delta_time);

                    delta_frame = time.elapsed().subsec_micros();
                }
                Event::LoopDestroyed => {
                    unsafe {
                        self.device
                            .device_wait_idle()
                            .expect("Failed to wait device idle!")
                    };
                }
                _ => (),
            }
        })
    }
}

impl Drop for VulkanApiObjects {
    fn drop(&mut self) {
        unsafe {
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.device
                    .destroy_semaphore(self.image_available_semaphores[i], None);
                self.device
                    .destroy_semaphore(self.render_finished_semaphores[i], None);
                self.device.destroy_fence(self.in_flight_fences[i], None);
            }
            self.cleanup_swapchain();
            self.device
                .destroy_descriptor_set_layout(self.ubo_layout, None);
            self.uniform_buffers.iter().enumerate().for_each(|(i, _)| {
                self.device.destroy_buffer(self.uniform_buffers[i], None);
                self.device
                    .free_memory(self.uniform_buffers_memory[i], None);
            });
            self.device.destroy_buffer(self.index_buffer, None);
            self.device.free_memory(self.index_buffer_memory, None);
            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_buffer_memory, None);
            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_device(None);
            self.surface
                .surface_loader
                .destroy_surface(self.surface.surface, None);
            if VALIDATION.is_enable {
                self.debug_utils_loader
                    .destroy_debug_utils_messenger(self.debug_messenger, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}
