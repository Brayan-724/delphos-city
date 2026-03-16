use std::borrow::Cow;
use std::collections::HashMap;

use delphos_ecs::{Resource, World};

use crate::{
    BindLayout, BindParams, Camera, Material, MaterialLayout, Shader, ShaderBuilder, ShaderId,
    ShaderModule, shader,
};

pub struct DelphosRenderRaw {
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
}

impl Default for DelphosRenderRaw {
    fn default() -> Self {
        panic!("DelphosRenderRaw should be inserted")
    }
}

impl Resource for DelphosRenderRaw {}

#[derive(Default)]
pub struct DelphosRender {
    pub named_shaders: HashMap<&'static str, ShaderId>,
}

impl Resource for DelphosRender {}

impl DelphosRender {
    pub fn create_shader<M: Material, W: World, S: shader::builder::State>(
        &mut self,
        world: &mut W,
        name: &'static str,
        source: Cow<'_, str>,
        shader: ShaderBuilder<'_, '_, '_, '_, '_, '_, '_, '_, '_, W, S>,
    ) -> ShaderId
    where
        S::Name: shader::builder::IsUnset,
        S::World: shader::builder::IsUnset,
        S::Device: shader::builder::IsUnset,
        S::BindLayouts: shader::builder::IsUnset,
        S::Shader: shader::builder::IsUnset,
        S::VertexBuffers: shader::builder::IsSet,
        S::FragmentFormat: shader::builder::IsSet,
        S::FragmentBlend: shader::builder::IsSet,
    {
        let raw_render = world.resource::<DelphosRenderRaw>().read();

        let shader_module = ShaderModule::new(
            &raw_render.device,
            source,
            Some(&format!("{name} shader source")),
        );

        let shader_module = world.spawn_component(shader_module);

        let bind_layout = {
            let mut binds = Vec::new();

            let mut binding = 0;
            while let Some(MaterialLayout {
                visibility,
                ty,
                count,
            }) = M::layout(binding)
            {
                binds.push(wgpu::BindGroupLayoutEntry {
                    binding,
                    visibility,
                    ty,
                    count,
                });
                binding += 1;
            }

            BindLayout::new(
                &raw_render.device,
                &binds,
                Some(&format!("{name} shader binds")),
            )
        };
        let bind_layout = world.spawn_component(bind_layout);

        let camera_layout = Camera::get(world)
            .read()
            .bind_group
            .get(world)
            .read()
            .layout;

        let shader = shader
            .name(name)
            .world(world)
            .device(&raw_render.device)
            .bind_layouts(&[bind_layout, camera_layout])
            .shader(shader_module)
            .build();
        world.spawn_component(shader)
    }
}
