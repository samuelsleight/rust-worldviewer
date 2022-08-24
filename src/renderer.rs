use stateloop::{
    app::{EventLoop, Window, WindowBuilder},
    winit::dpi::LogicalSize,
};
use std::sync::Arc;
use vulkano::{instance::Instance, swapchain::Surface};
use vulkano_win::{CreationError, VkSurfaceBuild};

pub struct Renderer {}

impl Renderer {
    pub fn construct_window(
        event_loop: &EventLoop<()>,
        instance: Arc<Instance>,
    ) -> Result<Arc<Surface<Window>>, CreationError> {
        WindowBuilder::new()
            .with_title("World Viewer")
            .with_inner_size(LogicalSize::new(1280, 720))
            .build_vk_surface(event_loop, instance)
    }

    pub fn init_vulkan(instance: Arc<Instance>, surface: &Arc<Surface<Window>>) -> Self {
        Self {}
    }
}
