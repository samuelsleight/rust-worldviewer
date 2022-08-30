use stateloop::app::{EventLoop, Window};
use std::{cell::RefCell, sync::Arc};
use vulkano::{
    buffer::{
        immutable::ImmutableBufferCreationError, BufferUsage, CpuBufferPool, ImmutableBuffer,
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

use self::{
    frame::{frame_state, RenderFrame},
    shaders::SceneData,
    vertex::Vertex,
};

mod frame;
mod init;
mod shaders;
mod vertex;

pub struct CoreObjects {
    device: Arc<Device>,
    queue: Arc<Queue>,
    swapchain: Arc<Swapchain<Window>>,
    images: Vec<Arc<SwapchainImage<Window>>>,
}

pub struct RendererData {
    objects: CoreObjects,

    vertex_buffer: Arc<ImmutableBuffer<[Vertex]>>,
    uniform_buffer: CpuBufferPool<SceneData>,
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
    UnableToCreateVertexBuffer(ImmutableBufferCreationError),
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
                    position: [100.0, 200.0],
                },
                Vertex {
                    position: [500.0, 150.0],
                },
                Vertex {
                    position: [400.0, 600.0],
                },
            ]
            .iter()
            .cloned(),
            BufferUsage::vertex_buffer(),
            objects.queue.clone(),
        )
        .map_err(InitError::UnableToCreateVertexBuffer)?;

        let uniform_buffer = CpuBufferPool::<SceneData>::uniform_buffer(objects.device.clone());

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
                uniform_buffer,
                pipeline,
                render_pass,
                framebuffers: None,

                viewport,
                frame_future: Some(Box::new(buffer_future)),
                recreate_swapchain: false,
            }),
        })
    }

    pub fn render<F>(&self, surface: &Arc<Surface<Window>>, frame_callback: F)
    where
        F: FnOnce(RenderFrame<frame_state::RenderPass>) -> RenderFrame<frame_state::Done>,
    {
        let mut data = self.data.borrow_mut();

        let mut frame_future = data.frame_future.take().unwrap();
        frame_future.cleanup_finished();

        let dimensions = surface.window().inner_size();

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

        let frame = RenderFrame::new(&mut data, image_num).begin(
            surface
                .window()
                .inner_size()
                .to_logical::<f32>(surface.window().scale_factor()),
        );

        let builder = frame_callback(frame).unwrap();
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
