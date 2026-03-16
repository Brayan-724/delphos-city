use delphos_ecs::{ComponentId, Resource, World};
use delphos_math::FVec2;
use wgpu::util::DeviceExt;

use crate::{BindGroup, BindGroupId, BindLayout, DelphosRenderRaw};

const Z_RANGE: f32 = 100.;

pub type CameraId = ComponentId<Camera>;

pub struct Camera {
    pub scale: f32,
    pub target: FVec2,
    pub viewport: FVec2,
    pub rendering: FVec2,
    pub bind_group: BindGroupId,
}

impl Default for Camera {
    fn default() -> Self {
        panic!("Camera should be inserted")
    }
}

impl Resource for Camera {}

impl Camera {
    pub fn new(world: &mut impl World, viewport: FVec2, rendering: FVec2) -> Self {
        let raw_render = world.resource::<DelphosRenderRaw>().read();

        let layout = BindLayout::new(
            &raw_render.device,
            &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            Some("Camera bind layout"),
        );
        let layout = world.spawn_component(layout);

        let matrix = get_matrix(1., rendering, FVec2::ZERO);
        let buffer = get_buffer(&raw_render.device, matrix);

        let bind_group = BindGroup::new(
            world,
            &raw_render.device,
            layout,
            &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            Some("Camera bind"),
        );
        let bind_group = world.spawn_component(bind_group);

        Self {
            scale: 1.,
            target: FVec2::ZERO,
            viewport,
            rendering,
            bind_group,
        }
    }

    pub fn to_world(&self, v: FVec2) -> FVec2 {
        // in physical pixels
        let p = v * self.rendering / self.viewport;
        (p - self.rendering * 0.5) / self.scale + self.target
    }

    pub fn get_bounds(&self) -> (FVec2, FVec2) {
        let half = self.rendering * 0.5 / self.scale;
        (self.target - half, self.target + half)
    }

    pub fn get_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        get_buffer(device, self.get_matrix())
    }

    pub fn update_buffer(&self, world: &mut impl World) {
        let raw_render = world.resource::<DelphosRenderRaw>();

        let buffer = get_buffer(&raw_render.read().device, self.get_matrix());

        world.component(&self.bind_group).write().update(
            world,
            &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        );
    }

    pub fn get_matrix(&self) -> [[f32; 4]; 4] {
        get_matrix(self.scale, self.rendering, self.target)
    }
}

fn get_buffer(device: &wgpu::Device, matrix: [[f32; 4]; 4]) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Camera buffer"),
        contents: bytemuck::cast_slice(&[matrix]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

fn get_matrix(scale: f32, rendering: FVec2, target: FVec2) -> [[f32; 4]; 4] {
    let zoom = 1. / scale;
    let n = -Z_RANGE;
    let f = Z_RANGE;

    let zoomed = rendering * zoom / 2.;

    let l = target.x - zoomed.x;
    let r = target.x + zoomed.x;
    let t = target.y - zoomed.y;
    let b = target.y + zoomed.y;

    [
        [2. / (r - l), 0., 0., 0.],
        [0., 2. / (b - t), 0., 0.],
        [0., 0., 1. / (f - n), 0.],
        [-(r + l) / (r - l), -(b + t) / (b - t), -n / (f - n), 1.],
    ]
}
