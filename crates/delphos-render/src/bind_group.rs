use delphos_ecs::{Component, ComponentId, World};

use crate::DelphosRenderRaw;

// ------ Bind Group ------

pub type BindGroupId = ComponentId<BindGroup>;
pub struct BindGroup {
    label: Option<String>,
    pub group: wgpu::BindGroup,
    pub layout: BindLayoutId,
}

impl Component for BindGroup {}

impl BindGroup {
    pub fn spawn<'a>(
        world: &mut impl World,
        layout: BindLayoutId,
        entries: &[wgpu::BindGroupEntry<'a>],
        label: Option<String>,
    ) -> BindGroupId {
        let group = Self::new(world, layout, entries, label);
        world.spawn_component(group)
    }

    pub fn new<'a>(
        world: &mut impl World,
        layout: BindLayoutId,
        entries: &[wgpu::BindGroupEntry<'a>],
        label: Option<String>,
    ) -> Self {
        let device = &world.resource::<DelphosRenderRaw>().read().device;

        Self {
            group: device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: label.as_ref().map(|s| s.as_str()),
                layout: &world.component(&layout).read().layout,
                entries,
            }),
            layout,
            label,
        }
    }

    pub fn update<'a>(&mut self, world: &mut impl World, entries: &[wgpu::BindGroupEntry<'a>]) {
        let raw_render = world.resource::<DelphosRenderRaw>().read();

        self.group = raw_render
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: self.label.as_ref().map(|s| s.as_str()),
                layout: &world.component(&self.layout).read().layout,
                entries,
            });
    }
}

// ------ Bind Group Layout ------

pub type BindLayoutId = ComponentId<BindLayout>;
pub struct BindLayout {
    pub layout: wgpu::BindGroupLayout,
}

impl Component for BindLayout {}

impl BindLayout {
    pub fn new<'a>(
        device: &wgpu::Device,
        entries: &[wgpu::BindGroupLayoutEntry],
        label: Option<&'a str>,
    ) -> Self {
        Self {
            layout: device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label, entries }),
        }
    }

    pub fn entry_texture(
        binding: u32,
        visibility: wgpu::ShaderStages,
    ) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            count: None,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
        }
    }

    pub fn entry_sampler(
        binding: u32,
        visibility: wgpu::ShaderStages,
    ) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        }
    }
}
