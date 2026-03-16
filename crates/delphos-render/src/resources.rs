use delphos_ecs::Resource;

use crate::BindParams;

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

pub struct DelphosRender {
    pub params: BindParams,
}

impl Default for DelphosRender {
    fn default() -> Self {
        panic!("DelphosRender should be inserted")
    }
}

impl Resource for DelphosRender {}
