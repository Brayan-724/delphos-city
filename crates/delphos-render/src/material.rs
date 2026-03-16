use std::num::NonZero;

use delphos_ecs::World;

use crate::{BindGroupId, ShaderId};

pub trait Material: Sized {
    fn layout(binding: u32) -> Option<MaterialLayout>;
    fn entry(&self, binding: u32) -> Option<wgpu::BindingResource<'_>>;

    fn bind(&self, world: &mut impl World, shader: ShaderId) -> BindGroupId {
        shader.get(world).read().create_material(world, self)
    }
}

pub struct MaterialLayout {
    pub visibility: wgpu::ShaderStages,
    pub ty: wgpu::BindingType,
    pub count: Option<NonZero<u32>>,
}

impl MaterialLayout {
    pub fn vertex(ty: wgpu::BindingType) -> Self {
        Self {
            visibility: wgpu::ShaderStages::VERTEX,
            ty,
            count: None,
        }
    }

    pub fn fragment(ty: wgpu::BindingType) -> Self {
        Self {
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty,
            count: None,
        }
    }

    pub const TEXTURE: wgpu::BindingType = wgpu::BindingType::Texture {
        sample_type: wgpu::TextureSampleType::Float { filterable: true },
        view_dimension: wgpu::TextureViewDimension::D2,
        multisampled: false,
    };

    pub const SAMPLER: wgpu::BindingType =
        wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering);

    pub fn texture<'a>(view: &'a wgpu::TextureView) -> wgpu::BindingResource<'a> {
        wgpu::BindingResource::TextureView(&view)
    }

    pub fn sampler<'a>(sampler: &'a wgpu::Sampler) -> wgpu::BindingResource<'a> {
        wgpu::BindingResource::Sampler(&sampler)
    }
}
