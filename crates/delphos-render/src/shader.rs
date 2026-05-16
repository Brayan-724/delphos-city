use std::any::{TypeId, type_name};
use std::borrow::Cow;
use std::marker::PhantomData;

use delphos_ecs::{Resource, ResourceId, World};
use wgpu::naga::FastHashMap;

use crate::{DelphosRender, DelphosRenderRaw, Material, ShaderMaterial};

pub type VertexBufferLayouts = &'static [wgpu::VertexBufferLayout<'static>];

pub trait Shader: Sized + 'static {
    const NAME: &'static str;
    const SHADER_ID: ShaderId = ResourceId::new::<ShaderModule, Self>();

    fn source() -> Cow<'static, str>;
    fn config(world: &mut impl World) -> ShaderModuleConfig;

    fn materials(materials: &mut ShaderMaterials<Self>);
}

type MaterialLayoutsFn = fn() -> Vec<wgpu::BindGroupLayoutEntry>;

pub struct ShaderMaterials<S: Shader> {
    materials: FastHashMap<TypeId, (usize, &'static str, MaterialLayoutsFn)>,
    _marker: PhantomData<S>,
}

impl<S: Shader> Default for ShaderMaterials<S> {
    fn default() -> Self {
        Self {
            materials: Default::default(),
            _marker: Default::default(),
        }
    }
}

impl<S: Shader> ShaderMaterials<S> {
    pub fn add_material<M: ShaderMaterial<S>>(&mut self) -> &mut Self {
        self.materials.insert(
            TypeId::of::<M>(),
            (M::BINDING, type_name::<M>(), M::layouts),
        );

        self
    }

    pub(crate) fn build(self, world: &mut impl World) -> Vec<(TypeId, wgpu::BindGroupLayout)> {
        let mut materials = self.materials.into_iter().collect::<Vec<_>>();

        materials.sort_by_key(|(_, (binding, _, _))| *binding);

        if cfg!(debug_assertions) {
            // FIXME: change to `array_windows`
            let are_contiguous = materials
                .windows(2)
                .all(|b| b[0].1.0.abs_diff(b[1].1.0) == 1);

            if !are_contiguous {
                panic!("Materials are not contiguous for shader {:?}", S::NAME);
            }
        }

        let mut layouts = vec![];

        let render_raw = world.resource::<DelphosRenderRaw>();
        let device = &render_raw.device;

        let mut render = world.resource::<DelphosRender>().write();

        for (mat, (_, mat_name, factory)) in materials {
            let layout = if let Some(layout) = render.bind_groups.get(&mat) {
                layout.clone()
            } else {
                let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some(&format!("{} bind layout {}", S::NAME, mat_name)),
                    entries: &(factory)(),
                });

                render.bind_groups.insert(mat, layout.clone());

                layout
            };

            layouts.push((mat, layout));
        }

        layouts
    }
}

pub type ShaderId = ResourceId;
pub struct ShaderModule {
    #[cfg(debug_assertions)]
    pub name: &'static str,
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub materials: Vec<TypeId>,
}

impl Resource for ShaderModule {}

#[derive(bon::Builder)]
pub struct ShaderModuleConfig {
    #[builder(default = "vs_main")]
    vertex_main: &'static str,
    vertex_buffers: Option<&'static [wgpu::VertexBufferLayout<'static>]>,
    #[builder(default)]
    vertex_compilation: wgpu::PipelineCompilationOptions<'static>,
    #[builder(default = "fs_main")]
    fragment_main: &'static str,
    #[builder(default)]
    fragment_compilation: wgpu::PipelineCompilationOptions<'static>,
    fragment_format: wgpu::TextureFormat,
    fragment_blend: Option<wgpu::BlendState>,
    #[builder(default = wgpu::ColorWrites::ALL)]
    fragment_write_mask: wgpu::ColorWrites,
    #[builder(default = wgpu::PrimitiveTopology::TriangleList)]
    topology: wgpu::PrimitiveTopology,
    strip_index_format: Option<wgpu::IndexFormat>,
    #[builder(default = wgpu::FrontFace::Ccw)]
    front_face: wgpu::FrontFace,
    cull_mode: Option<wgpu::Face>,
    #[builder(default = wgpu::PolygonMode::Fill)]
    polygon_mode: wgpu::PolygonMode,
    #[builder(default = false)]
    unclipped_depth: bool,
    #[builder(default = false)]
    conservative: bool,
}

impl ShaderModule {
    pub fn new<S: Shader>(world: &mut impl World) -> Self {
        let name = S::NAME;

        let config = S::config(world);

        let mut materials = ShaderMaterials::<S>::default();
        S::materials(&mut materials);
        let materials = materials.build(world);

        let raw_render = world.resource::<DelphosRenderRaw>().read();
        let device = &raw_render.device;

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{name} pipeline layout")),
            bind_group_layouts: &materials.iter().map(|m| &m.1).collect::<Vec<_>>(),
            immediate_size: 0,
        });

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("{name} shader source")),
            source: wgpu::ShaderSource::Wgsl(S::source()),
        });

        // https://sotrh.github.io/learn-wgpu/beginner/tutorial3-pipeline/#how-do-we-use-the-shaders
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("{name} pipeline")),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some(config.vertex_main),
                buffers: config.vertex_buffers.unwrap_or_default(),
                compilation_options: config.vertex_compilation,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some(config.fragment_main),
                compilation_options: config.fragment_compilation,
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.fragment_format,
                    blend: config.fragment_blend,
                    write_mask: config.fragment_write_mask,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: config.topology,
                strip_index_format: config.strip_index_format,
                front_face: config.front_face,
                cull_mode: config.cull_mode,
                polygon_mode: config.polygon_mode,
                unclipped_depth: config.unclipped_depth,
                conservative: config.conservative,
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
            #[cfg(debug_assertions)]
            name,
            pipeline,
            materials: materials.into_iter().map(|(m, _)| m).collect(),
        }
    }

    pub fn binding<M: Material>(&self) -> Option<usize> {
        self.materials.iter().position(|m| *m == TypeId::of::<M>())
    }
}

pub trait WorldShading: World {
    fn register_shader<S: Shader>(&mut self) -> &mut Self {
        let shader_module = ShaderModule::new::<S>(self);

        self.insert_resource(shader_module);

        self
    }
}

impl<W: World> WorldShading for W {}
