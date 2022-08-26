use stateloop::{
    app::{EventLoop, Window},
    winit::dpi::PhysicalSize,
};
use std::{cell::RefCell, sync::Arc};
use vulkano::{
    buffer::{
        immutable::ImmutableBufferCreationError, BufferUsage, ImmutableBuffer, TypedBufferAccess,
    },
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassContents,
    },
    device::{physical::SurfacePropertiesError, Device, DeviceCreationError, Queue},
    image::{view::ImageView, ImageAccess, SwapchainImage},
    instance::Instance,
    pipeline::{
        graphics::{
            input_assembly::InputAssemblyState,
            vertex_input::BuffersDefinition,
            viewport::{Viewport, ViewportState},
            GraphicsPipelineCreationError,
        },
        GraphicsPipeline,
    },
    render_pass::{
        Framebuffer, FramebufferCreateInfo, RenderPass, RenderPassCreationError, Subpass,
    },
    shader::ShaderCreationError,
    single_pass_renderpass,
    swapchain::{
        acquire_next_image, AcquireError, Surface, Swapchain, SwapchainCreateInfo,
        SwapchainCreationError,
    },
    sync::{now, FlushError, GpuFuture},
};
use vulkano_win::CreationError;

use self::vertex::Vertex;

mod init;
mod shaders;
mod vertex;

pub struct CoreObjects {
    device: Arc<Device>,
    queue: Arc<Queue>,
    swapchain: Arc<Swapchain<Window>>,
    images: Vec<Arc<SwapchainImage<Window>>>,
}

struct RendererData {
    objects: CoreObjects,

    vertex_buffer: Arc<ImmutableBuffer<[Vertex]>>,
    pipeline: Arc<GraphicsPipeline>,
    render_pass: Arc<RenderPass>,
    framebuffers: Option<Vec<Arc<Framebuffer>>>,

    viewport: Viewport,
    frame_future: Option<Box<dyn GpuFuture>>,
    recreate_swapchain: bool,
}

pub struct Renderer {
    data: RefCell<RendererData>,
}

#[derive(Debug)]
pub enum InitError {
    NoSuitableDeviceFound,
    UnableToCreateDevice(DeviceCreationError),
    UnableToGetSurfaceCapabilities(SurfacePropertiesError),
    UnableToGetSurfaceFormats(SurfacePropertiesError),
    UnableToCreateSwapchain(SwapchainCreationError),
    UnableToCreateRenderPass(RenderPassCreationError),
    UnableToCreatePipeline(GraphicsPipelineCreationError),
    UnableToLoadShaders(ShaderCreationError),
    UnableToCreateBuffer(ImmutableBufferCreationError),
}

impl Renderer {
    pub fn construct_window(
        event_loop: &EventLoop<()>,
        instance: Arc<Instance>,
    ) -> Result<Arc<Surface<Window>>, CreationError> {
        init::construct_window(event_loop, instance)
    }

    pub fn init_vulkan(
        instance: &Arc<Instance>,
        surface: &Arc<Surface<Window>>,
    ) -> Result<Self, InitError> {
        let objects = init::init_core_objects(instance, surface)?;

        let render_pass = single_pass_renderpass!(
            objects.device.clone(),
            attachments:{
                colour: {
                    load: Clear,
                    store: Store,
                    format: objects.swapchain.image_format(),
                    samples: 1,
                }
            },
            pass: {
                color: [colour],
                depth_stencil: {}
            }
        )
        .map_err(InitError::UnableToCreateRenderPass)?;

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [0.0, 0.0],
            depth_range: 0.0..1.0,
        };

        let shaders =
            shaders::load(objects.device.clone()).map_err(InitError::UnableToLoadShaders)?;

        let (vertex_buffer, buffer_future) = ImmutableBuffer::from_iter(
            [
                Vertex {
                    position: [-0.5, -0.25],
                },
                Vertex {
                    position: [0.0, 0.5],
                },
                Vertex {
                    position: [0.25, -0.3],
                },
            ]
            .iter()
            .cloned(),
            BufferUsage::vertex_buffer(),
            objects.queue.clone(),
        )
        .map_err(InitError::UnableToCreateBuffer)?;

        let pipeline = GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
            .vertex_shader(shaders.vertex.entry_point("main").unwrap(), ())
            .input_assembly_state(InputAssemblyState::new())
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .fragment_shader(shaders.fragment.entry_point("main").unwrap(), ())
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(objects.device.clone())
            .map_err(InitError::UnableToCreatePipeline)?;

        Ok(Self {
            data: RefCell::new(RendererData {
                objects,

                vertex_buffer,
                pipeline,
                render_pass,
                framebuffers: None,

                viewport,
                frame_future: Some(Box::new(buffer_future)),
                recreate_swapchain: false,
            }),
        })
    }

    pub fn render(&self, dimensions: PhysicalSize<u32>) {
        let mut data = self.data.borrow_mut();

        let mut frame_future = data.frame_future.take().unwrap();
        frame_future.cleanup_finished();

        if data.recreate_swapchain {
            let (swapchain, images) = match data.objects.swapchain.recreate(SwapchainCreateInfo {
                image_extent: dimensions.into(),
                ..data.objects.swapchain.create_info()
            }) {
                Ok(result) => result,
                Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                Err(e) => panic!("{:?}", e),
            };

            data.objects.swapchain = swapchain;
            data.objects.images = images;
            data.framebuffers = None;
            data.recreate_swapchain = false;
        }

        if data.framebuffers.is_none() {
            let [w, h] = data.objects.images[0].dimensions().width_height();
            data.viewport.dimensions = [w as f32, h as f32];

            let framebuffers = data
                .objects
                .images
                .iter()
                .map(|image| {
                    let view = ImageView::new_default(image.clone()).unwrap();

                    Framebuffer::new(
                        data.render_pass.clone(),
                        FramebufferCreateInfo {
                            attachments: vec![view],
                            ..Default::default()
                        },
                    )
                    .unwrap()
                })
                .collect::<Vec<_>>();

            data.framebuffers = Some(framebuffers);
        }

        let (image_num, suboptimal, acquire_future) =
            match acquire_next_image(data.objects.swapchain.clone(), None) {
                Ok(result) => result,
                Err(AcquireError::OutOfDate) => {
                    data.recreate_swapchain = true;
                    return;
                }
                Err(e) => panic!("{:?}", e),
            };

        if suboptimal {
            data.recreate_swapchain = true;
        }

        let mut builder = AutoCommandBufferBuilder::primary(
            data.objects.device.clone(),
            data.objects.queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([1.0, 0.0, 1.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(
                        data.framebuffers.as_ref().unwrap()[image_num].clone(),
                    )
                },
                SubpassContents::Inline,
            )
            .unwrap()
            .set_viewport(0, [data.viewport.clone()])
            .bind_pipeline_graphics(data.pipeline.clone())
            .bind_vertex_buffers(0, data.vertex_buffer.clone())
            .draw(data.vertex_buffer.len() as u32, 1, 0, 0)
            .unwrap()
            .end_render_pass()
            .unwrap();

        let command_buffer = builder.build().unwrap();

        let future = frame_future
            .join(acquire_future)
            .then_execute(data.objects.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                data.objects.queue.clone(),
                data.objects.swapchain.clone(),
                image_num,
            )
            .then_signal_fence_and_flush();

        let end_future = match future {
            Ok(future) => Box::new(future) as Box<_>,
            Err(FlushError::OutOfDate) => {
                data.recreate_swapchain = true;
                Box::new(now(data.objects.device.clone())) as Box<_>
            }
            Err(_) => Box::new(now(data.objects.device.clone())) as Box<_>,
        };

        data.frame_future = Some(end_future);
    }
}
