use delphos_ecs::{ComponentId, Resource, World};
use delphos_math::FVec2;
use wgpu::util::DeviceExt;

use crate::{DelphosRenderRaw, Material, MaterialLayout};

const Z_RANGE: f32 = 100.;

pub type CameraId = ComponentId<Camera>;

pub struct Camera {
    pub scale: f32,
    pub target: FVec2,
    pub viewport: FVec2,
    pub rendering: FVec2,
    buffer: wgpu::Buffer,
}

impl Default for Camera {
    fn default() -> Self {
        panic!("Camera should be inserted")
    }
}

impl Resource for Camera {}

impl Material for Camera {
    fn layout(binding: u32) -> Option<MaterialLayout> {
        match binding {
            0 => Some(MaterialLayout::vertex_fragment(wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            })),
            _ => None,
        }
    }

    fn update(&mut self, world: &mut impl World) {
        let raw_render = world.resource::<DelphosRenderRaw>().read();

        self.buffer = self.get_buffer(&raw_render.device);
    }

    fn entry(&self, binding: u32) -> Option<wgpu::BindingResource<'_>> {
        match binding {
            0 => Some(self.buffer.as_entire_binding()),
            _ => None,
        }
    }
}

impl Camera {
    pub fn new(world: &mut impl World, viewport: FVec2, rendering: FVec2) -> Self {
        let raw_render = world.resource::<DelphosRenderRaw>().read();

        Self {
            scale: 1.,
            target: FVec2::ZERO,
            viewport,
            rendering,
            buffer: get_buffer(&raw_render.device, [[0.; 4]; 4]),
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
