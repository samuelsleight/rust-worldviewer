use bytemuck::{Pod, Zeroable};
use vulkano::impl_vertex;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Zeroable, Pod)]
pub struct Vertex {
    pub position: [f32; 2],
    pub texture: [f32; 2],
}

impl_vertex!(Vertex, position, texture);

impl Vertex {
    pub fn new(x: f32, y: f32, u: f32, v: f32) -> Self {
        Self {
            position: [x, y],
            texture: [u, v],
        }
    }
}
