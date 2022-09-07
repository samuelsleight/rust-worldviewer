use stateloop::winit::dpi::LogicalSize;
use vulkano::{
    command_buffer::{
        pool::standard::{StandardCommandPoolAlloc, StandardCommandPoolBuilder},
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
    },
};

use super::{
    material::{Material, MaterialParams},
    RendererData,
};

pub mod frame_state {
    use crate::renderer::material::{Material, MaterialParams};

    pub struct Begin;
    pub struct RenderPass<'mat, Params: MaterialParams> {
        pub material: &'mat Material<Params>,
    }
    pub struct Done;
}

pub trait ReadyForMaterial {}

impl ReadyForMaterial for frame_state::Begin {}
impl<'mat, Params: MaterialParams> ReadyForMaterial for frame_state::RenderPass<'mat, Params> {}

pub struct RenderFrame<'data, State> {
    data: &'data mut RendererData,
    image_num: usize,
    window_size: LogicalSize<f32>,
    builder: AutoCommandBufferBuilder<
        PrimaryAutoCommandBuffer<StandardCommandPoolAlloc>,
        StandardCommandPoolBuilder,
    >,
    state: State,
}

impl<'data> RenderFrame<'data, frame_state::Begin> {
    pub fn new(
        data: &'data mut RendererData,
        image_num: usize,
        window_size: LogicalSize<f32>,
    ) -> Self {
        let builder = AutoCommandBufferBuilder::primary(
            data.objects.device.clone(),
            data.objects.queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        Self {
            data,
            image_num,
            window_size,
            builder,
            state: frame_state::Begin,
        }
    }
}

impl<'data, State> RenderFrame<'data, State>
where
    State: ReadyForMaterial,
{
    pub fn material<Params: MaterialParams>(
        mut self,
        material: &Material<Params>,
    ) -> RenderFrame<'data, frame_state::RenderPass<Params>> {
        let uniform_buffer = self
            .data
            .uniform_buffer
            .next(self.window_size.into())
            .unwrap();
        material.bind(
            &self.data,
            &mut self.builder,
            self.image_num,
            uniform_buffer,
        );

        RenderFrame {
            data: self.data,
            image_num: self.image_num,
            window_size: self.window_size,
            builder: self.builder,
            state: frame_state::RenderPass { material },
        }
    }
}

impl<'data, 'mat, Params: MaterialParams>
    RenderFrame<'data, frame_state::RenderPass<'mat, Params>>
{
    pub fn draw(mut self, position: LogicalSize<f32>, render_data: Params::RenderData) -> Self {
        Params::update(&self.state.material, &mut self.builder, position, render_data);

        self.builder.draw(4, 1, 0, 0).unwrap();
        self
    }

    pub fn finish(mut self) -> RenderFrame<'data, frame_state::Done> {
        self.builder.end_render_pass().unwrap();

        RenderFrame {
            data: self.data,
            image_num: self.image_num,
            window_size: self.window_size,
            builder: self.builder,
            state: frame_state::Done,
        }
    }
}

impl<'data> RenderFrame<'data, frame_state::Done> {
    pub fn unwrap(
        self,
    ) -> AutoCommandBufferBuilder<
        PrimaryAutoCommandBuffer<StandardCommandPoolAlloc>,
        StandardCommandPoolBuilder,
    > {
        self.builder
    }
}
