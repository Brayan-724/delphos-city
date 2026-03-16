use std::ffi::c_void;
use std::ptr::NonNull;

pub use ::wgpu;
use delphos_ecs::World;
use delphos_math::UVec2;
use wgpu::rwh::{RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle};

pub use self::bind_group::*;
pub use self::render_queue::*;
pub use self::resources::*;
pub use self::shader::*;
pub use self::structs::*;

mod bind_group;
mod render_queue;
mod resources;
mod shader;
mod structs;

pub unsafe fn display_handle(ptr: NonNull<c_void>) -> RawDisplayHandle {
    RawDisplayHandle::Wayland(WaylandDisplayHandle::new(ptr))
}

pub unsafe fn window_handle(ptr: NonNull<c_void>) -> RawWindowHandle {
    RawWindowHandle::Wayland(WaylandWindowHandle::new(ptr))
}

pub async fn start<W: World>(
    world: &mut W,
    raw_display_handle: RawDisplayHandle,
    raw_window_handle: RawWindowHandle,
) {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let surface = unsafe {
        instance
            .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle,
                raw_window_handle,
            })
            .expect("Failed to create surface")
    };

    // Pick a supported adapter
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            power_preference: wgpu::PowerPreference::LowPower,
            ..Default::default()
        })
        .await
        .expect("Failed to find suitable adapter");

    let (device, queue) = adapter
        .request_device(&wgpu::wgt::DeviceDescriptor {
            required_features: wgpu::Features::POLYGON_MODE_LINE,
            ..Default::default()
        })
        .await
        .expect("Failed to request device");

    world.insert_resource(DelphosRenderRaw {
        adapter,
        device,
        queue,
        surface,
    });
}

pub fn configure<W: World>(world: &mut W, size: UVec2) {
    let raw_render = world.resource::<DelphosRenderRaw>().read();

    let surface = &raw_render.surface;
    let device = &raw_render.device;

    let cap = surface.get_capabilities(&raw_render.adapter);
    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: cap.formats[0],
        view_formats: vec![cap.formats[0]],
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        width: size.x,
        height: size.y,
        desired_maximum_frame_latency: 2,
        present_mode: wgpu::PresentMode::Fifo,
    };

    surface.configure(&device, &surface_config);

    let shader_module = ShaderModule::new(
        &device,
        include_str!("../../../assets/shaders/base.wgsl").into(),
        Some("Base shader"),
    );

    let shader_module = world.spawn_component(shader_module);

    let bind_layout = BindLayout::new(
        &device,
        &[
            BindLayout::entry_texture(0, wgpu::ShaderStages::FRAGMENT),
            BindLayout::entry_sampler(1, wgpu::ShaderStages::FRAGMENT),
        ],
        Some("Texture Binds"),
    );
    let bind_layout = world.spawn_component(bind_layout);

    let shader = Shader::builder()
        .name("Base")
        .world(world)
        .device(&device)
        .bind_layouts(&[bind_layout])
        .shader(shader_module)
        .vertex_buffers(&[Vertex::layout()])
        .fragment_format(cap.formats[0])
        .fragment_blend(wgpu::BlendState::ALPHA_BLENDING)
        .build();
    let shader = world.spawn_component(shader);

    let image = asefile::AsepriteFile::read(&include_bytes!("../../../assets/person.aseprite")[..])
        .unwrap()
        .frame(0)
        .image();

    let texture = create_texture(world, image);

    let sampler = device.create_sampler(&Default::default());
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let material = BindGroup::spawn(
        world,
        &device,
        bind_layout,
        &[
            BindGroup::entry_texture(0, &view),
            BindGroup::entry_sampler(1, &sampler),
        ],
        Some("Base bind group"),
    );

    let params = BindParams { shader, material };

    world.insert_resource(DelphosRender { params });
}

pub fn draw<W: World>(world: &mut W) {
    let raw_render = world.resource::<DelphosRenderRaw>().read();
    let render_queue = world.resource::<RenderQueue>();

    let surface = &raw_render.surface;
    let device = &raw_render.device;
    let queue = &raw_render.queue;

    // We don't plan to render much in this example, just clear the surface.
    let surface_texture = surface
        .get_current_texture()
        .expect("failed to acquire next swapchain texture");
    let texture_view = surface_texture
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            ..Default::default()
        });

        if render_queue.read().len() != 0 {
            let (vertex_buffer, index_buffer) = render_queue.read().buffers(device);

            rp.set_vertex_buffer(0, vertex_buffer.slice(..));
            rp.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            let mut offset = 0;
            let mut batch_start = 0;
            let mut current_params = render_queue.read().triangles()[0].params;

            let pipeline = world.component(&current_params.shader);
            rp.set_pipeline(&pipeline.read().pipeline);

            let material = world.component(&current_params.material);
            rp.set_bind_group(0, &material.read().group, &[]);

            for tri in render_queue.read().triangles() {
                let end = offset + 3;

                if current_params != tri.params {
                    // draw the previous batch first
                    rp.draw_indexed(batch_start..offset, 0, 0..1);

                    if current_params.shader != tri.params.shader {
                        let pipeline = world.component(&tri.params.shader);
                        rp.set_pipeline(&pipeline.read().pipeline);
                    }
                    if current_params.material != tri.params.material {
                        let material = world.component(&tri.params.material);
                        rp.set_bind_group(0, &material.read().group, &[]);
                    }
                    current_params = tri.params;
                    batch_start = offset;
                }
                offset = end;
            }
            rp.draw_indexed(batch_start..offset, 0, 0..1);
        }
    }

    render_queue.write().clear();

    // Submit the command in the queue to execute
    queue.submit(Some(encoder.finish()));
    surface_texture.present();
}

pub fn create_texture(world: &mut impl World, image: image::RgbaImage) -> wgpu::Texture {
    let raw_render = world.resource::<DelphosRenderRaw>().read();

    let size = wgpu::Extent3d {
        width: image.width(),
        height: image.height(),
        depth_or_array_layers: 1,
    };

    let texture = raw_render
        .device
        .create_texture(&wgpu::wgt::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

    raw_render.queue.write_texture(
        texture.as_image_copy(),
        &image.into_vec(),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(texture.width() * 4),
            rows_per_image: None,
        },
        texture.size(),
    );

    texture
}
