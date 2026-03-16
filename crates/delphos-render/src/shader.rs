use std::borrow::Cow;

use delphos_ecs::{Component, ComponentId, World};

use crate::{BindGroup, BindGroupId, BindLayout, BindLayoutId, DelphosRenderRaw, Material};

// ------ Shader ------

pub type ShaderId = ComponentId<Shader>;
pub struct Shader {
    pub name: String,
    pub module: ShaderModuleId,
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub bind_layout: BindLayoutId,
}

impl Component for Shader {}

pub mod builder {
    pub use super::shader_builder::*;
}

#[bon::bon]
impl Shader {
    #[builder]
    pub fn new(
        world: &mut impl World,
        #[builder(into)] name: String,
        device: &wgpu::Device,
        shader: ShaderModuleId,
        bind_layouts: &[BindLayoutId],
        #[builder(default = "vs_main")] vertex_main: &str,
        vertex_buffers: Option<&[wgpu::VertexBufferLayout<'_>]>,
        #[builder(default)] vertex_compilation: wgpu::PipelineCompilationOptions<'_>,
        #[builder(default = "fs_main")] fragment_main: &str,
        #[builder(default)] fragment_compilation: wgpu::PipelineCompilationOptions<'_>,
        fragment_format: wgpu::TextureFormat,
        fragment_blend: Option<wgpu::BlendState>,
        #[builder(default = wgpu::ColorWrites::ALL)] fragment_write_mask: wgpu::ColorWrites,
        #[builder(default = wgpu::PrimitiveTopology::TriangleList)]
        topology: wgpu::PrimitiveTopology,
        strip_index_format: Option<wgpu::IndexFormat>,
        #[builder(default = wgpu::FrontFace::Ccw)] front_face: wgpu::FrontFace,
        cull_mode: Option<wgpu::Face>,
        #[builder(default = wgpu::PolygonMode::Fill)] polygon_mode: wgpu::PolygonMode,
        #[builder(default = false)] unclipped_depth: bool,
        #[builder(default = false)] conservative: bool,
    ) -> Self {
        let main_bind = if let Some(bind) = bind_layouts.get(0) {
            *bind
        } else {
            let layout = BindLayout::new(device, &[], Some(&format!("{name} binds layout")));
            world.spawn_component(layout)
        };

        let pipeline_layout = {
            let label = format!("{name} pipeline layout");

            let mut layouts = Vec::with_capacity(bind_layouts.len());

            for layout in bind_layouts {
                layouts.push(world.component(layout).read());
            }

            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&label),
                bind_group_layouts: &layouts.iter().map(|l| &l.layout).collect::<Vec<_>>(),
                immediate_size: 0,
            })
        };

        let shader_res = world.component(&shader).read();

        // https://sotrh.github.io/learn-wgpu/beginner/tutorial3-pipeline/#how-do-we-use-the-shaders
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("{name} pipeline")),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_res.module,
                entry_point: Some(vertex_main),
                buffers: vertex_buffers.unwrap_or_default(),
                compilation_options: vertex_compilation,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_res.module,
                entry_point: Some(fragment_main),
                compilation_options: fragment_compilation,
                targets: &[Some(wgpu::ColorTargetState {
                    format: fragment_format,
                    blend: fragment_blend,
                    write_mask: fragment_write_mask,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology,
                strip_index_format,
                front_face,
                cull_mode,
                polygon_mode,
                unclipped_depth,
                conservative,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        Self {
            name,
            module: shader,
            pipeline,
            bind_layout: main_bind,
        }
    }
}

impl Shader {
    pub fn create_material(&self, world: &mut impl World, material: &impl Material) -> BindGroupId {
        let mut entries = Vec::new();

        let mut binding = 0;
        while let Some(resource) = material.entry(binding) {
            entries.push(wgpu::BindGroupEntry { binding, resource });
            binding += 1;
        }

        BindGroup::spawn(
            world,
            self.bind_layout,
            &entries,
            Some(format!("{} binds", self.name)),
        )
    }
}

// ------ Shader Module ------

pub type ShaderModuleId = ComponentId<ShaderModule>;
pub struct ShaderModule {
    pub module: wgpu::ShaderModule,
}

impl Component for ShaderModule {}

impl ShaderModule {
    pub fn new<'a>(device: &wgpu::Device, source: Cow<'a, str>, label: Option<&'a str>) -> Self {
        Self {
            module: device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label,
                source: wgpu::ShaderSource::Wgsl(source),
            }),
        }
    }
}
