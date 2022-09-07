use std::sync::Arc;

use stateloop::winit::dpi::LogicalSize;
use vulkano::{
    buffer::BufferAccess,
    command_buffer::{
        pool::standard::{StandardCommandPoolAlloc, StandardCommandPoolBuilder},
        AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassContents,
    },
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    device::Device,
    image::ImageViewAbstract,
    pipeline::{
        graphics::{
            color_blend::ColorBlendState,
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            vertex_input::BuffersDefinition,
            viewport::ViewportState,
        },
        GraphicsPipeline, Pipeline, PipelineBindPoint,
    },
    render_pass::Subpass,
};

use super::{shaders::VertexConstants, vertex::Vertex, InitError, RendererData};

pub use texture::TextureMaterial;

mod texture;

pub unsafe trait MaterialParams {
    type Data;

    fn construct_data(&self, device: &Arc<Device>) -> Self::Data;

    fn update(
        material: &Material<Self>,
        builder: &mut AutoCommandBufferBuilder<
            PrimaryAutoCommandBuffer<StandardCommandPoolAlloc>,
            StandardCommandPoolBuilder,
        >,
        position: LogicalSize<f32>,
        texture: Arc<dyn ImageViewAbstract>,
    );
}

pub struct Material<Params: MaterialParams + ?Sized> {
    data: Params::Data,
    pipeline: Arc<GraphicsPipeline>,
}

impl<Params: MaterialParams> Material<Params> {
    pub fn new(renderer: &RendererData, params: Params) -> Result<Self, InitError> {
        let subpass = Subpass::from(renderer.render_pass.clone(), 0).unwrap();

        let pipeline = GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
            .vertex_shader(
                renderer.shaders.vertex.entry_point("main").unwrap(),
                VertexConstants { quad_scale: 300.0 },
            )
            .input_assembly_state(
                InputAssemblyState::new().topology(PrimitiveTopology::TriangleStrip),
            )
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .fragment_shader(renderer.shaders.fragment.entry_point("main").unwrap(), ())
            .color_blend_state(ColorBlendState::new(subpass.num_color_attachments()).blend_alpha())
            .render_pass(subpass)
            .build(renderer.objects.device.clone())
            .map_err(InitError::UnableToCreatePipeline)?;

        Ok(Self {
            data: params.construct_data(&renderer.objects.device),
            pipeline,
        })
    }

    pub fn bind(
        &self,
        renderer: &RendererData,
        builder: &mut AutoCommandBufferBuilder<
            PrimaryAutoCommandBuffer<StandardCommandPoolAlloc>,
            StandardCommandPoolBuilder,
        >,
        image_num: usize,
        uniform_buffer: Arc<dyn BufferAccess>,
    ) {
        let descriptor_set = PersistentDescriptorSet::new(
            self.pipeline.layout().set_layouts().get(0).unwrap().clone(),
            [WriteDescriptorSet::buffer(0, uniform_buffer)],
        )
        .unwrap();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([1.0, 0.0, 1.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(
                        renderer.framebuffers.as_ref().unwrap()[image_num].clone(),
                    )
                },
                SubpassContents::Inline,
            )
            .unwrap()
            .set_viewport(0, [renderer.viewport.clone()])
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .bind_vertex_buffers(0, renderer.vertex_buffer.clone());
    }
}
