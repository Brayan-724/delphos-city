use std::any::{TypeId, type_name};
use std::collections::HashMap;

use delphos_ecs::{Resource, World};

use crate::{Camera, MaterialBind, Shader, ShaderId, ShaderModule};

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
    /// TypeId: Material
    pub bind_groups: HashMap<TypeId, wgpu::BindGroupLayout>,

    pub default_camera: Option<MaterialBind<Camera>>,
}

impl Resource for DelphosRender {}
