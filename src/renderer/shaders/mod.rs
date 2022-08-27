use std::sync::Arc;

use vulkano::{
    device::Device,
    shader::{ShaderCreationError, ShaderModule},
};

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/renderer/shaders/vertex.glsl",
        types_meta: {
            use bytemuck::{Pod, Zeroable};

            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/renderer/shaders/fragment.glsl",
    }
}

pub struct Shaders {
    pub vertex: Arc<ShaderModule>,
    pub fragment: Arc<ShaderModule>,
}

pub use vs::ty::SceneData;

pub fn load(device: Arc<Device>) -> Result<Shaders, ShaderCreationError> {
    Ok(Shaders {
        vertex: vs::load(device.clone())?,
        fragment: fs::load(device.clone())?,
    })
}
