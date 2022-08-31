use stateloop::winit::dpi::LogicalSize;
use std::marker::PhantomData;
use vulkano::{
    command_buffer::{
        pool::standard::{StandardCommandPoolAlloc, StandardCommandPoolBuilder},
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
        RenderPassBeginInfo, SubpassContents,
    },
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    pipeline::{Pipeline, PipelineBindPoint},
};

use super::{shaders::MeshData, RendererData};

pub mod frame_state {
    pub struct Begin;
    pub struct RenderPass;
    pub struct Done;
}

pub struct RenderFrame<'data, State> {
    data: &'data mut RendererData,
    image_num: usize,
    builder: AutoCommandBufferBuilder<
        PrimaryAutoCommandBuffer<StandardCommandPoolAlloc>,
        StandardCommandPoolBuilder,
    >,
    _marker: PhantomData<State>,
}

impl<'data> RenderFrame<'data, frame_state::Begin> {
    pub fn new(data: &'data mut RendererData, image_num: usize) -> Self {
        let builder = AutoCommandBufferBuilder::primary(
            data.objects.device.clone(),
            data.objects.queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        Self {
            data,
            image_num,
            builder,
            _marker: PhantomData,
        }
    }

    pub fn begin(
        mut self,
        window_size: LogicalSize<f32>,
    ) -> RenderFrame<'data, frame_state::RenderPass> {
        let uniform_buffer = self.data.uniform_buffer.next(window_size.into()).unwrap();

        let descriptor_set = PersistentDescriptorSet::new(
            self.data
                .pipeline
                .layout()
                .set_layouts()
                .get(0)
                .unwrap()
                .clone(),
            [
                WriteDescriptorSet::buffer(0, uniform_buffer),
                WriteDescriptorSet::image_view_sampler(
                    1,
                    self.data.texture.clone(),
                    self.data.sampler.clone(),
                ),
            ],
        )
        .unwrap();

        self.builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([1.0, 0.0, 1.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(
                        self.data.framebuffers.as_ref().unwrap()[self.image_num].clone(),
                    )
                },
                SubpassContents::Inline,
            )
            .unwrap()
            .set_viewport(0, [self.data.viewport.clone()])
            .bind_pipeline_graphics(self.data.pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.data.pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .bind_vertex_buffers(0, self.data.vertex_buffer.clone());

        RenderFrame {
            data: self.data,
            builder: self.builder,
            image_num: self.image_num,
            _marker: PhantomData,
        }
    }
}

impl<'data> RenderFrame<'data, frame_state::RenderPass> {
    pub fn draw(mut self, position: LogicalSize<f32>) -> Self {
        self.builder
            .push_constants(
                self.data.pipeline.layout().clone(),
                0,
                MeshData {
                    offset: position.into(),
                },
            )
            .draw(4, 1, 0, 0)
            .unwrap();

        RenderFrame {
            data: self.data,
            builder: self.builder,
            image_num: self.image_num,
            _marker: PhantomData,
        }
    }

    pub fn finish(mut self) -> RenderFrame<'data, frame_state::Done> {
        self.builder.end_render_pass().unwrap();

        RenderFrame {
            data: self.data,
            builder: self.builder,
            image_num: self.image_num,
            _marker: PhantomData,
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
