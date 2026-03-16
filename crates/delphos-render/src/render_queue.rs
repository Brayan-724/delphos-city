use delphos_ecs::{ComponentId, Resource};
use wgpu::util::DeviceExt;

use crate::{BindParams, Triangle, Vertex};

pub type RenderQueueId = ComponentId<RenderQueue>;

#[derive(Default)]
pub struct RenderQueue {
    vertices: Vec<Vertex>,
    triangles: Vec<Triangle>,
}

impl Resource for RenderQueue {}

impl RenderQueue {
    pub fn len(&self) -> usize {
        self.triangles.len()
    }

    pub fn triangles(&self) -> &[Triangle] {
        &self.triangles
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.triangles.clear();
    }

    pub fn add_to_queue(
        &mut self,
        vertices: &[Vertex],
        indices: &[u16],
        z_index: i32,
        params: BindParams,
    ) {
        if indices.len() % 3 != 0 {
            log::warn!("Indices are not divisible by 3");
        }

        let offset = self.vertices.len() as u16;
        self.vertices.extend(vertices);

        let idx = self.triangles.partition_point(|a| {
            a.z_index
                .cmp(&z_index)
                .then(a.params.shader.cmp(&params.shader))
                .then(a.params.material.cmp(&params.material))
                .is_lt()
        });

        let right = self.triangles.split_off(idx);
        self.triangles.extend(indices.chunks(3).map(|v| Triangle {
            indices: [v[0] + offset, v[1] + offset, v[2] + offset],
            z_index,
            params,
        }));
        self.triangles.extend(right);
    }

    /// (vertex_buffer, index_buffer)
    pub fn buffers(&self, device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer) {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let indices = self
            .triangles
            .iter()
            .map(|t| t.indices)
            .flatten()
            .collect::<Vec<_>>();

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        (vertex_buffer, index_buffer)
    }
}
