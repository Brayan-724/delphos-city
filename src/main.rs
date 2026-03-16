use delphos_ecs::World;
use delphos_math::{FVec2, IVec2, UVec2};
use delphos_render::{DelphosRender, Vertex};
use delphos_window::sctk::{Region, WaylandSurface};
use delphos_window::{
    DelphosWindowApp, DelphosWindowDraw, DelphosWindowKeyboard, DelphosWindowPointer, OpenWindow,
    Time, sctk,
};

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_module("naga", log::LevelFilter::Info)
        .filter_module("wgpu_hal", log::LevelFilter::Info)
        .filter_module("sctk", log::LevelFilter::Info)
        .init();

    delphos_window::start_window::<DelphosApp>().await;
}

struct DelphosApp {
    size: UVec2,
}

impl DelphosWindowApp for DelphosApp {
    fn setup(_: IVec2, size: IVec2, output: delphos_window::wayland::WlOutput) -> OpenWindow {
        OpenWindow::builder()
            .anchor(sctk::Anchor::BOTTOM)
            .namespace("delphos-city")
            .output(output)
            .size(size.as_u32().set_y(100))
            .build()
    }

    type NewError = ();
    fn new(world: &mut delphos_window::DelphosWorld) -> Result<Self, Self::NewError> {
        let empty_region = Region::new(&world.window.compositor).unwrap();

        world
            .window
            .layer_surface
            .set_input_region(Some(empty_region.wl_region()));
        world.window.layer_surface.wl_surface().commit();

        Ok(Self {
            size: UVec2::default(),
        })
    }

    fn configure(
        &mut self,
        _: &mut delphos_window::DelphosWorld,
        _: &delphos_window::ConfigureCtx<'_, Self>,
        configure: sctk::LayerSurfaceConfigure,
        _: u32,
    ) {
        self.size = UVec2::from(configure.new_size);
    }
}

impl DelphosWindowKeyboard for DelphosApp {
    fn press_key(
        &mut self,
        world: &mut delphos_window::DelphosWorld,
        _: delphos_window::KeyboardCtx<'_, Self>,
        event: sctk::KeyEvent,
    ) {
        if event.keysym == sctk::Keysym::Escape {
            world.exit()
        }
    }
}

impl DelphosWindowPointer for DelphosApp {}

impl DelphosWindowDraw for DelphosApp {
    fn draw(
        &mut self,
        world: &mut delphos_window::DelphosWorld,
        ctx: delphos_window::DrawCtx<'_, Self>,
    ) {
        let elapsed = world.resource::<Time>().read().elapsed;

        const COUNT: u32 = 17;
        for i in 0..COUNT {
            const TIME: u32 = 10000;
            const PADDING: f32 = 1.;
            const PADDED: u32 = TIME + (TIME as f32 * PADDING) as u32;

            let elapsed = elapsed + i * (PADDED / COUNT);
            let offset = ((elapsed % PADDED) as f32) / TIME as f32 - PADDING;
            draw_person(world, FVec2::new(offset * 6. - 1., 0.25));
        }

        // Request our next frame
        world
            .window
            .layer_surface
            .wl_surface()
            .frame(ctx.qh, world.window.layer_surface.wl_surface().clone());

        world.window.layer_surface.commit();
    }
}

fn draw_person(world: &mut impl World, position: FVec2) {
    let x = -1.35 + position.x * 2.;
    let y = -1.0 + position.y * 2.;

    let vertices = &[
        Vertex {
            position: [0. + x, 0. + y, 0.],
            color: [1., 1., 1., 1.],
            uv: [0., 1.],
        },
        Vertex {
            position: [1. + x, 0. + y, 0.],
            color: [1., 1., 1., 1.],
            uv: [1., 1.],
        },
        Vertex {
            position: [1. + x, 1. + y, 0.],
            color: [1., 1., 1., 1.],
            uv: [1., 0.],
        },
        Vertex {
            position: [0. + x, 1. + y, 0.],
            color: [1., 1., 1., 1.],
            uv: [0., 0.],
        },
    ];

    let indices: &[u16] = &[0, 1, 2, 0, 2, 3];

    world
        .resource::<delphos_render::RenderQueue>()
        .write()
        .add_to_queue(
            vertices,
            indices,
            0,
            world.resource::<DelphosRender>().read().params,
        );
}
