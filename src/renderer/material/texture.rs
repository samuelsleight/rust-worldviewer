use std::sync::Arc;

use vulkano::{
    command_buffer::{
        pool::standard::{StandardCommandPoolAlloc, StandardCommandPoolBuilder},
        AutoCommandBufferBuilder, PrimaryAutoCommandBuffer,
    },
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    pipeline::{Pipeline, PipelineBindPoint},
    sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo}, image::ImageViewAbstract,
};

use crate::renderer::shaders::MeshData;

use super::{Material, MaterialParams};

pub struct TextureMaterial;

pub struct TextureData {
    sampler: Arc<Sampler>,
}

unsafe impl MaterialParams for TextureMaterial {
    type PipelineData = TextureData;
    type RenderData = Arc<dyn ImageViewAbstract>;

    fn construct_data(&self, device: &Arc<vulkano::device::Device>) -> Self::PipelineData {
        let sampler = Sampler::new(
            device.clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                address_mode: [SamplerAddressMode::ClampToEdge; 3],
                ..Default::default()
            },
        )
        .unwrap();

        TextureData { sampler }
    }

    fn update(
        material: &Material<Self>,
        builder: &mut AutoCommandBufferBuilder<
            PrimaryAutoCommandBuffer<StandardCommandPoolAlloc>,
            StandardCommandPoolBuilder,
        >,
        position: stateloop::winit::dpi::LogicalSize<f32>,
        render_data: Self::RenderData,
    ) {
        let texture = render_data;

        let descriptor_set = PersistentDescriptorSet::new(
            material
                .pipeline
                .layout()
                .set_layouts()
                .get(1)
                .unwrap()
                .clone(),
            [WriteDescriptorSet::image_view_sampler(
                0,
                texture,
                material.data.sampler.clone(),
            )],
        )
        .unwrap();

        builder
            .push_constants(
                material.pipeline.layout().clone(),
                0,
                MeshData {
                    offset: position.into(),
                },
            )
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                material.pipeline.layout().clone(),
                1,
                descriptor_set,
            );
    }
}
