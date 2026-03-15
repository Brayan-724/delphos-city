use delphos_ecs::WorldContainer;
use delphos_math::U32Vec2;
use wayland_client::globals::registry_queue_init;

use crate::{
    DelphosWindow, DelphosWindowDraw, DelphosWindowKeyboard, DelphosWindowPointer,
    DelphosWindowState, outputs, sctk, wayland,
};

pub fn start_window<State: DelphosWindowDraw + DelphosWindowPointer + DelphosWindowKeyboard>() {
    let conn = wayland::Connection::connect_to_env().unwrap();
    let (pos, size, output) = outputs::get_main_output(&conn).unwrap();

    State::setup(pos, size, output).run::<State>(&conn);
}

#[derive(bon::Builder)]
pub struct OpenWindow {
    anchor: sctk::Anchor,
    #[builder(default = sctk::KeyboardInteractivity::None)]
    keyboard: sctk::KeyboardInteractivity,
    #[builder(default = sctk::Layer::Top)]
    layer: sctk::Layer,
    #[builder(into)]
    namespace: Option<String>,
    size: U32Vec2,
    output: Option<wayland::WlOutput>,
}

impl OpenWindow {
    fn create_layer_surface<
        State: DelphosWindowDraw + DelphosWindowPointer + DelphosWindowKeyboard,
    >(
        self,
        globals: &wayland::GlobalList,
        qh: &wayland::QueueHandle<DelphosWindow<State>>,
    ) -> (sctk::CompositorState, sctk::LayerSurface) {
        use sctk::WaylandSurface;

        let compositor =
            sctk::CompositorState::bind(globals, qh).expect("wl_compositor is not available");
        let layer_shell =
            sctk::LayerShell::bind(globals, qh).expect("layer shell is not available");

        // A layer surface is created from a surface.
        let surface = compositor.create_surface(qh);

        let layer_surface = layer_shell.create_layer_surface(
            qh,
            surface,
            self.layer,
            self.namespace,
            self.output.as_ref(),
        );
        layer_surface.set_size(self.size.x, self.size.y);
        layer_surface.set_anchor(self.anchor);
        layer_surface.set_keyboard_interactivity(self.keyboard);
        layer_surface.set_exclusive_zone(-1);
        {
            let empty_region = sctk::Region::new(&compositor).unwrap();
            layer_surface.set_input_region(Some(empty_region.wl_region()));
        }
        layer_surface.wl_surface().commit();

        (compositor, layer_surface)
    }

    pub fn run<State: DelphosWindowDraw + DelphosWindowPointer + DelphosWindowKeyboard>(
        self,
        conn: &wayland::Connection,
    ) {
        let (globals, mut event_queue) = registry_queue_init(conn).unwrap();
        let qh = event_queue.handle();

        let shm = sctk::Shm::bind(&globals, &qh).expect("wl_shm is not available");

        let (compositor, layer_surface) = self.create_layer_surface(&globals, &qh);

        let pool = sctk::SlotPool::new(256 * 100 * 4, &shm).expect("Failed to create pool");

        let mut window_state = DelphosWindowState {
            compositor,
            registry: sctk::RegistryState::new(&globals),
            seat: sctk::SeatState::new(&globals, &qh),
            output: sctk::OutputState::new(&globals, &qh),
            shm,
            layer_surface,
            pool,

            world: WorldContainer::default(),

            exit: false,
            configured: false,
        };

        let mut layer = DelphosWindow {
            app: State::new(&mut window_state).expect("Cannot intiantiate app"),
            window: window_state,
        };

        event_queue.roundtrip(&mut layer).unwrap();

        loop {
            event_queue.blocking_dispatch(&mut layer).unwrap();

            if layer.window.exit {
                log::warn!("Exiting");
                break;
            }
        }
    }
}
