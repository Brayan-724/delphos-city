use std::any::{TypeId, type_name};
use std::num::NonZero;

use delphos_ecs::{Resource, UntypedResourceId, ResourceId, World};
use wgpu::BindGroupLayoutEntry;

use crate::{DelphosRender, DelphosRenderRaw, Shader};

pub trait Material: Sized + 'static {
    const MATERIAL_ID: MaterialBindId = MaterialBindId::new::<Self>();

    fn layout(binding: u32) -> Option<MaterialLayout>;
    #[expect(unused_variables)]
    fn update(&mut self, world: &mut impl World) {}
    fn entry(&self, binding: u32) -> Option<wgpu::BindingResource<'_>>;

    fn layouts() -> Vec<BindGroupLayoutEntry> {
        let mut binds = Vec::new();

        let mut binding = 0;
        while let Some(layout) = Self::layout(binding) {
            binds.push(layout.binded(binding));
            binding += 1;
        }

        binds
    }

    fn bind(&mut self, world: &mut impl World) -> MaterialBind {
        MaterialBind::new(self, world)
    }
}

pub trait ShaderMaterial<S: Shader>: Material {
    const BINDING: usize;

    fn bind_shaded(&mut self, world: &mut impl World) -> MaterialBind {
        MaterialBind::new_shaded::<S, Self>(self, world)
    }
}

pub struct MaterialLayout {
    pub visibility: wgpu::ShaderStages,
    pub ty: wgpu::BindingType,
    pub count: Option<NonZero<u32>>,
}

impl MaterialLayout {
    pub fn binded(self, binding: u32) -> wgpu::BindGroupLayoutEntry {
        let MaterialLayout {
            visibility,
            ty,
            count,
        } = self;

        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty,
            count,
        }
    }

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

    pub fn vertex_fragment(ty: wgpu::BindingType) -> Self {
        Self {
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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

pub type MaterialBindId = ResourceId<MaterialBind>;
pub struct MaterialBind {
    label: String,
    pub group: wgpu::BindGroup,
}

impl Resource for MaterialBind {}

impl MaterialBind {
    pub fn new_raw<M: Material>(label: String, material: &mut M, world: &mut impl World) -> Self {
        let entries = &Self::get_entries(material, world);

        let device = &world.resource::<DelphosRenderRaw>().device;
        let layout = &world.resource::<DelphosRender>().bind_groups[&TypeId::of::<M>()];

        Self {
            group: device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&label),
                layout,
                entries,
            }),
            label,
        }
    }

    pub fn new<M: Material>(material: &mut M, world: &mut impl World) -> Self {
        let label = format!("{} bind", type_name::<M>());
        Self::new_raw(label, material, world)
    }

    pub fn new_shaded<S: Shader, M: ShaderMaterial<S>>(
        material: &mut M,
        world: &mut impl World,
    ) -> Self {
        let label = format!("{} bind ({}) for {}", type_name::<M>(), M::BINDING, S::NAME);
        Self::new_raw(label, material, world)
    }

    pub fn get_entries<'a>(
        material: &'a mut impl Material,
        world: &mut impl World,
    ) -> Vec<wgpu::BindGroupEntry<'a>> {
        material.update(world);

        let mut entries = Vec::new();
        let mut binding = 0;
        while let Some(resource) = material.entry(binding) {
            entries.push(wgpu::BindGroupEntry { binding, resource });
            binding += 1;
        }

        entries
    }

    pub fn update<M: Material>(&mut self, material: &mut M, world: &mut impl World) {
        let entries = &Self::get_entries(material, world);

        let raw_render = world.resource::<DelphosRenderRaw>().read();
        let layout = &world.resource::<DelphosRender>().bind_groups[&TypeId::of::<M>()];

        self.group = raw_render
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&self.label),
                layout,
                entries,
            });
    }
}
