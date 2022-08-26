use std::sync::Arc;

use stateloop::{
    app::{EventLoop, Window, WindowBuilder},
    winit::dpi::LogicalSize,
};
use vulkano::{
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType},
        Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo,
    },
    image::ImageUsage,
    instance::Instance,
    swapchain::{Surface, Swapchain, SwapchainCreateInfo},
};
use vulkano_win::{CreationError, VkSurfaceBuild};

use super::{CoreObjects, InitError};

pub fn construct_window(
    event_loop: &EventLoop<()>,
    instance: Arc<Instance>,
) -> Result<Arc<Surface<Window>>, CreationError> {
    WindowBuilder::new()
        .with_title("World Viewer")
        .with_inner_size(LogicalSize::new(1280, 720))
        .build_vk_surface(event_loop, instance)
}

pub fn init_core_objects(
    instance: &Arc<Instance>,
    surface: &Arc<Surface<Window>>,
) -> Result<CoreObjects, InitError> {
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::none()
    };

    let (physical_device, queue_family) = PhysicalDevice::enumerate(instance)
        .filter(|&device| {
            device
                .supported_extensions()
                .is_superset_of(&device_extensions)
        })
        .filter_map(|device| {
            device
                .queue_families()
                .find(|&queue| {
                    queue.supports_graphics() && queue.supports_surface(surface).unwrap_or(false)
                })
                .map(|queue| (device, queue))
        })
        .min_by_key(|(device, _)| match device.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            PhysicalDeviceType::Other => 4,
        })
        .ok_or(InitError::NoSuitableDeviceFound)?;

    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            enabled_extensions: device_extensions,
            queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
            ..Default::default()
        },
    )
    .map_err(InitError::UnableToCreateDevice)?;

    let queue = queues.next().unwrap();

    let (swapchain, images) = {
        let surface_capabilities = physical_device
            .surface_capabilities(surface, Default::default())
            .map_err(InitError::UnableToGetSurfaceCapabilities)?;

        let image_format = Some(
            physical_device
                .surface_formats(surface, Default::default())
                .map_err(InitError::UnableToGetSurfaceFormats)?[0]
                .0,
        );

        Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: surface_capabilities.min_image_count,
                image_format,
                image_extent: surface.window().inner_size().into(),
                image_usage: ImageUsage::color_attachment(),
                composite_alpha: surface_capabilities
                    .supported_composite_alpha
                    .iter()
                    .next()
                    .unwrap(),
                ..Default::default()
            },
        )
        .map_err(InitError::UnableToCreateSwapchain)?
    };

    Ok(CoreObjects {
        device,
        queue,
        swapchain,
        images,
    })
}
