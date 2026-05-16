use delphos_ecs::UntypedComponentId;

use crate::{MaterialBindId, Shader, ShaderId, ShaderMaterial};

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}

impl Vertex {
    const ATTRS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x4,
        2 => Float32x2,
    ];
    pub const fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRS,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Triangle {
    pub indices: [u16; 3],
    pub z_index: i32,
    pub params: BindParams,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BindParams {
    pub shader: ShaderId,
    pub material: MaterialBindId,
}

impl BindParams {
    pub fn new<S: Shader>(material: MaterialBindId<impl ShaderMaterial<S>>) -> Self {
        Self {
            shader: S::SHADER_ID,
            material: material.untyped(),
        }
    }
}
