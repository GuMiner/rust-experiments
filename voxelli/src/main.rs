#[macro_use]
extern crate vulkano;

#[macro_use]
extern crate vulkano_shader_derive;

extern crate vulkano_win;
extern crate winit;

use std::iter;
use std::fs::File;
use std::mem;
use std::sync::Arc;
use std::time::Duration;
use std::thread;

// Needed for build_vk_surface to bring the trait in scope
use vulkano_win::VkSurfaceBuild;

use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::device::Device;
use vulkano::framebuffer::Framebuffer;
use vulkano::framebuffer::Subpass;
use vulkano::instance::Instance;
use vulkano::instance::PhysicalDevice;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain;
use vulkano::swapchain::PresentMode;
use vulkano::swapchain::SurfaceTransform;
use vulkano::swapchain::Swapchain;
use vulkano::swapchain::AcquireError;
use vulkano::swapchain::SwapchainCreationError;
use vulkano::sync::now;
use vulkano::sync::GpuFuture;

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}

impl_vertex!(Vertex, position);

fn main() {
    let extensions = vulkano_win::required_extensions();
    let instance = Instance::new(None, &extensions, None)
        .expect("Failed to create the Vulkan instance for display.");

    let mut device_count = 0;
    for device in PhysicalDevice::enumerate(&instance) {
        println!(
            "Found device '{}' of type '{:?}'.",
            device.name(),
            device.ty()
        );
        device_count += 1;
    }

    let physical_device = PhysicalDevice::enumerate(&instance).next().unwrap();

    println!(
        "Enumerated {} device{}.",
        device_count,
        if device_count == 1 { "" } else { "s" }
    );

    let mut events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new()
        .build_vk_surface(&events_loop, instance.clone())
        .unwrap();

    let mut dimensions = {
        let (width, height) = window.window().get_inner_size_pixels().unwrap();
        [width, height]
    };

    let requested_queue = physical_device
        .queue_families()
        .find(|&queue| {
            queue.supports_graphics() && window.surface().is_supported(queue).unwrap_or(false)
        })
        .expect("Failed to find a graphics queue for sending drawing commands to.");

    let device_extensions = vulkano::device::DeviceExtensions {
        khr_swapchain: true,
        ..vulkano::device::DeviceExtensions::none()
    };

    let (device, mut queues) = Device::new(
        physical_device,
        physical_device.supported_features(),
        &device_extensions,
        [(requested_queue, 0.5)].iter().cloned(),
    ).expect(("Failed to create the Vulkan device!"));

    let graphics_queue = queues.next().unwrap();

    let capabilities = window
        .surface()
        .capabilities(physical_device)
        .expect("Failed to get the surface capabilities of the device");

    let alpha = capabilities
        .supported_composite_alpha
        .iter()
        .next()
        .unwrap();

    let format = capabilities.supported_formats[0].0;

    let (mut swapchain, mut images) = Swapchain::new(
        device.clone(),
        window.surface().clone(),
        capabilities.min_image_count,
        format,
        dimensions,
        1,
        capabilities.supported_usage_flags,
        &graphics_queue,
        SurfaceTransform::Identity,
        alpha,
        PresentMode::Fifo,
        true,
        None,
    ).expect("Failed to create the swapchain");

    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        [
            Vertex {
                position: [-0.5, -0.25],
            },
            Vertex {
                position: [0.0, 0.5],
            },
            Vertex {
                position: [0.25, -0.1],
            },
        ].iter()
            .cloned(),
    ).expect("Failed to creat ethe triangle vertex buffer");

    // TODO: Get off of VulkanoShader to allow file-based loading of shaders.
    // let mut vertexShader = String::new();
    // {
    //     let vertexShaderFile = File::open("shaders/demo.vs").expect("Could not open the vertex shader for reading!");
    //     vertexShaderFile.read_to_string(&mut vertexShader).expect("Could not read all of the vertex shader file!");
    // }

    // let mut fragmentShader = String::new();
    // {
    //     let fragmentShaderFile = File::open("shaders/demo.fs").expect("Could not open the fragment shader for reading!");
    //     fragmentShaderFile.read_to_string(&mut vertexShader).expect("Could not read all of the fragment shader file!");
    // }

    mod vs {
        #[derive(VulkanoShader)]
        #[ty = "vertex"]
        #[src = "
#version 450
layout(location = 0) in vec2 position;
void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}
"]
        struct Dummy;
    }

    mod fs {
        #[derive(VulkanoShader)]
        #[ty = "fragment"]
        #[src = "
#version 450
layout(location = 0) out vec4 f_color;
void main() {
    f_color = vec4(1.0, 0.0, 0.0, 1.0);
}
"]
        struct Dummy;
    }

    let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
    let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");

    let render_pass = Arc::new(
        single_pass_renderpass!(device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain.format(),
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }).unwrap(),
    );

    let pipeline = Arc::new(
        GraphicsPipeline::start()
        // We need to indicate the layout of the vertices.
        // The type `SingleBufferDefinition` actually contains a template parameter corresponding
        // to the type of each vertex. But in this code it is automatically inferred.
        .vertex_input_single_buffer()
        // A Vulkan shader can in theory contain multiple entry points, so we have to specify
        // which one. The `main` word of `main_entry_point` actually corresponds to the name of
        // the entry point.
        .vertex_shader(vs.main_entry_point(), ())
        // The content of the vertex buffer describes a list of triangles.
        .triangle_list()
        // Use a resizable viewport set to draw over the entire window
        .viewports_dynamic_scissors_irrelevant(1)
        // See `vertex_shader`.
        .fragment_shader(fs.main_entry_point(), ())
        // We have to indicate which subpass of which render pass this pipeline is going to be used
        // in. The pipeline will only be usable from this particular subpass.
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        // Now that our builder is filled, we call `build()` to obtain an actual pipeline.
        .build(device.clone())
        .unwrap(),
    );

    let mut framebuffers: Option<Vec<Arc<vulkano::framebuffer::Framebuffer<_, _>>>> = None;

    let mut recreate_swapchain = false;
    let mut previous_frame_end = Box::new(now(device.clone())) as Box<GpuFuture>;

    let mut frame = 0;
    loop {
        frame += 1;
        println!("Frame: {}", frame);

        // It is important to call this function from time to time, otherwise resources will keep
        // accumulating and you will eventually reach an out of memory error.
        // Calling this function polls various fences in order to determine what the GPU has
        // already processed, and frees the resources that are no longer needed.
        previous_frame_end.cleanup_finished();

        if recreate_swapchain {
            // Get the new dimensions for the viewport/framebuffers.
            dimensions = {
                let (new_width, new_height) = window.window().get_inner_size_pixels().unwrap();
                [new_width, new_height]
            };

            let (new_swapchain, new_images) = match swapchain.recreate_with_dimension(dimensions) {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::UnsupportedDimensions) => {
                    continue;
                }
                Err(err) => panic!("{:?}", err),
            };

            mem::replace(&mut swapchain, new_swapchain);
            mem::replace(&mut images, new_images);

            framebuffers = None;

            recreate_swapchain = false;
        }

        // Because framebuffers contains an Arc on the old swapchain, we need to
        // recreate framebuffers as well.
        if framebuffers.is_none() {
            let new_framebuffers = Some(
                images
                    .iter()
                    .map(|image| {
                        Arc::new(
                            Framebuffer::start(render_pass.clone())
                                .add(image.clone())
                                .unwrap()
                                .build()
                                .unwrap(),
                        )
                    })
                    .collect::<Vec<_>>(),
            );
            mem::replace(&mut framebuffers, new_framebuffers);
        }

        let (image_num, acquire_future) =
            match swapchain::acquire_next_image(swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    recreate_swapchain = true;
                    continue;
                }
                Err(err) => panic!("{:?}", err),
            };

        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), graphics_queue.family()).unwrap()
            // Before we can draw, we have to *enter a render pass*. There are two methods to do
            // this: `draw_inline` and `draw_secondary`. The latter is a bit more advanced and is
            // not covered here.
            //
            // The third parameter builds the list of values to clear the attachments with. The API
            // is similar to the list of attachments when building the framebuffers, except that
            // only the attachments that use `load: Clear` appear in the list.
            .begin_render_pass(framebuffers.as_ref().unwrap()[image_num].clone(), false,
                               vec![[0.0, 0.0, 1.0, 1.0].into()])
            .unwrap()

            // We are now inside the first subpass of the render pass. We add a draw command.
            //
            // The last two parameters contain the list of resources to pass to the shaders.
            // Since we used an `EmptyPipeline` object, the objects have to be `()`.
            .draw(pipeline.clone(),
                  DynamicState {
                      line_width: None,
                      // TODO: Find a way to do this without having to dynamically allocate a Vec every frame.
                      viewports: Some(vec![Viewport {
                          origin: [0.0, 0.0],
                          dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                          depth_range: 0.0 .. 1.0,
                      }]),
                      scissors: None,
                  },
                  vertex_buffer.clone(), (), ())
            .unwrap()

            // We leave the render pass by calling `draw_end`. Note that if we had multiple
            // subpasses we could have called `next_inline` (or `next_secondary`) to jump to the
            // next subpass.
            .end_render_pass()
            .unwrap()

            // Finish building the command buffer by calling `build`.
            .build().unwrap();

        let future = previous_frame_end.join(acquire_future)
            .then_execute(graphics_queue.clone(), command_buffer).unwrap()

            // The color output is now expected to contain our triangle. But in order to show it on
            // the screen, we have to *present* the image by calling `present`.
            //
            // This function does not actually present the image immediately. Instead it submits a
            // present command at the end of the queue. This means that it will only be presented once
            // the GPU has finished executing the command buffer that draws the triangle.
            .then_swapchain_present(graphics_queue.clone(), swapchain.clone(), image_num)
            .then_signal_fence_and_flush().unwrap();
        previous_frame_end = Box::new(future) as Box<_>;

        // Note that in more complex programs it is likely that one of `acquire_next_image`,
        // `command_buffer::submit`, or `present` will block for some time. This happens when the
        // GPU's queue is full and the driver has to wait until the GPU finished some work.
        //
        // Unfortunately the Vulkan API doesn't provide any way to not wait or to detect when a
        // wait would happen. Blocking may be the desired behavior, but if you don't want to
        // block you should spawn a separate thread dedicated to submissions.

        // Handling the window events in order to close the program when the user wants to close
        // it.
        let mut done = false;
        events_loop.poll_events(|ev| match ev {
            winit::Event::WindowEvent {
                event: winit::WindowEvent::Closed,
                ..
            } => done = true,
            winit::Event::WindowEvent {
                event: winit::WindowEvent::Resized(_, _),
                ..
            } => recreate_swapchain = true,
            _ => (),
        });
        if done {
            return;
        }
    }

    thread::sleep(Duration::from_millis(10000));
}
