use std::ffi::c_void;
use std::ptr::NonNull;

pub use ::wgpu;
use delphos_ecs::Resource;
use delphos_ecs::World;
use delphos_math::FVec2;
use delphos_math::UVec2;
use wgpu::rwh::{RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle};

pub use self::camera::*;
pub use self::material::*;
pub use self::render_queue::*;
pub use self::resources::*;
pub use self::shader::*;
pub use self::structs::*;

mod camera;
mod material;
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

    let mut camera = {
        let viewport = size.as_f32();
        let height = 3.; // GOOD Height
        // let height = 1.; // Debug Height

        Camera::new(
            world,
            viewport,
            FVec2::new(viewport.x / viewport.y * height, height),
        )
    };
    world.resource::<DelphosRender>().write().default_camera = Some(camera.bind(world));
    world.insert_resource(camera);

    struct BaseShader;

    impl Shader for BaseShader {
        const NAME: &'static str = "Base";

        fn source() -> std::borrow::Cow<'static, str> {
            include_str!("../../../assets/shaders/base.wgsl").into()
        }

        fn config(world: &mut impl World) -> ShaderModuleConfig {
            let raw_render = world.resource::<DelphosRenderRaw>().read();
            let cap = raw_render.surface.get_capabilities(&raw_render.adapter);

            static VERTEX_BUFFERS: VertexBufferLayouts = &[Vertex::layout()];

            ShaderModuleConfig::builder()
                .vertex_buffers(VERTEX_BUFFERS)
                .fragment_format(cap.formats[0])
                .fragment_blend(wgpu::BlendState::ALPHA_BLENDING)
                .build()
        }

        fn materials(materials: &mut ShaderMaterials<Self>) {
            materials.add_material::<BaseMaterial>();
        }
    }

    impl ShaderMaterial<BaseShader> for BaseMaterial {
        const BINDING: usize = 0;
    }

    impl ShaderMaterial<BaseShader> for Camera {
        const BINDING: usize = 1;
    }

    struct BaseMaterial {
        sampler: wgpu::Sampler,
        view: wgpu::TextureView,
    }

    impl Material for BaseMaterial {
        fn layout(binding: u32) -> Option<MaterialLayout> {
            match binding {
                0 => Some(MaterialLayout::fragment(MaterialLayout::TEXTURE)),
                1 => Some(MaterialLayout::fragment(MaterialLayout::SAMPLER)),
                _ => None,
            }
        }

        fn entry(&self, binding: u32) -> Option<wgpu::BindingResource<'_>> {
            match binding {
                0 => Some(MaterialLayout::texture(&self.view)),
                1 => Some(MaterialLayout::sampler(&self.sampler)),
                _ => None,
            }
        }
    }

    world.register_shader::<BaseShader>();

    let image = asefile::AsepriteFile::read(&include_bytes!("../../../assets/person.aseprite")[..])
        .unwrap()
        .frame(0)
        .image();

    let texture = create_texture(world, image);

    let sampler = device.create_sampler(&Default::default());
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut material = BaseMaterial { sampler, view };
    let bind = material.bind_shaded(world);
}

pub fn draw<W: World>(world: &mut W) {
    let raw_render = world.resource::<DelphosRenderRaw>().read();
    let render = world.resource::<DelphosRender>().read();
    let render_queue = world.resource::<RenderQueue>();

    let Some(camera) = render.default_camera else {
        log::warn!("No camera setup");
        return;
    };

    camera.update(&mut *Camera::get(world).write(), world);

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
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
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

            {
                let shader = current_params.shader.get::<ShaderModule>(world).read();
                rp.set_pipeline(&shader.pipeline);
            }

            {
                rp.set_bind_group(0, &current_params.material.get(world).read().group, &[]);
            }

            if let Some(binding) = shader.binding::<Camera>() {
                rp.set_bind_group(binding as u32, &camera.group, &[]);
            }

            for tri in render_queue.read().triangles() {
                let end = offset + 3;

                if current_params != tri.params {
                    // draw the previous batch first
                    rp.draw_indexed(batch_start..offset, 0, 0..1);

                    if current_params.shader != tri.params.shader {
                        let pipeline = tri.params.shader.get(world);
                        rp.set_pipeline(&pipeline.read().pipeline);
                    }
                    if current_params.material != tri.params.material {
                        let material = tri.params.material.get(world);
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
