use std::fmt;

use delphos_math::{IVec2, U32Vec2};
use wayland_client::globals::registry_queue_init;

mod boilerplate;
mod macros;
mod outputs;
mod window;

pub mod sctk {
    pub use smithay_client_toolkit::compositor::{CompositorState, Region};
    pub use smithay_client_toolkit::output::OutputState;
    pub use smithay_client_toolkit::registry::RegistryState;
    pub use smithay_client_toolkit::seat::{
        Capability, SeatState,
        keyboard::{KeyEvent, Keysym, Modifiers, RawModifiers},
        pointer::{AxisScroll, PointerEvent, PointerEventKind},
    };
    pub use smithay_client_toolkit::shell::WaylandSurface;
    pub use smithay_client_toolkit::shell::wlr_layer::{
        Anchor, KeyboardInteractivity, Layer, LayerShell, LayerSurface, LayerSurfaceConfigure,
    };
    pub use smithay_client_toolkit::shm::Shm;
    pub use smithay_client_toolkit::shm::slot::SlotPool;
}

pub mod wayland {
    pub use wayland_client::globals::GlobalList;
    pub use wayland_client::protocol::{
        wl_keyboard::WlKeyboard,
        wl_output::{Transform, WlOutput},
        wl_pointer::{AxisSource, WlPointer},
        wl_seat::WlSeat,
        wl_shm::Format as WlFormat,
        wl_surface::WlSurface,
    };
    pub use wayland_client::{Connection, QueueHandle};
}

pub struct DelphosWindow<State: DelphosWindowApp> {
    pub window: DelphosWindowState,
    pub app: State,
}

pub struct DelphosWindowState {
    pub compositor: sctk::CompositorState,
    pub registry: sctk::RegistryState,
    pub seat: sctk::SeatState,
    pub output: sctk::OutputState,
    pub shm: sctk::Shm,
    pub pool: sctk::SlotPool,
    pub layer_surface: sctk::LayerSurface,

    pub frame_count: u32,
    last_frame_count: u32,
    update_frame_count: u32,
    pub delta: u32,
    last_delta: u32,
    average_delta: u32,
    pub exit: bool,
    configured: bool,
}

pub type ConfigureCtx<'a, State> = boilerplate::LayerHandlerCtx<'a, DelphosWindow<State>>;

pub trait DelphosWindowApp: Sized + 'static {
    fn setup(pos: IVec2, size: IVec2, output: wayland::WlOutput) -> OpenWindow;

    type NewError: fmt::Debug;
    fn new(window: &mut DelphosWindowState) -> Result<Self, Self::NewError>;

    fn configure(
        &mut self,
        window: &mut DelphosWindowState,
        ctx: &ConfigureCtx<'_, Self>,
        configure: sctk::LayerSurfaceConfigure,
        serial: u32,
    );
}

// ------- Keyboard -------

pub type KeyboardCtx<'a, State> = boilerplate::KeyboardHandlerCtx<'a, DelphosWindow<State>>;

#[expect(unused_variables, reason = "Blank implementations")]
pub trait DelphosWindowKeyboard: DelphosWindowApp {
    fn press_key(
        &mut self,
        window: &mut DelphosWindowState,
        ctx: KeyboardCtx<'_, Self>,
        event: sctk::KeyEvent,
    ) {
    }
    fn repeat_key(
        &mut self,
        window: &mut DelphosWindowState,
        ctx: KeyboardCtx<'_, Self>,
        event: sctk::KeyEvent,
    ) {
    }
    fn release_key(
        &mut self,
        window: &mut DelphosWindowState,
        ctx: KeyboardCtx<'_, Self>,
        event: sctk::KeyEvent,
    ) {
    }
}

// ------- Pointer -------

pub type PointerCtx<'a, State> = boilerplate::PointerHandlerCtx<'a, DelphosWindow<State>>;
pub type PointerEventCtx<'a, State> = boilerplate::PointerEventHandlerCtx<'a, DelphosWindow<State>>;

#[expect(unused_variables, reason = "Blank implementations")]
pub trait DelphosWindowPointer: DelphosWindowApp {
    fn pointer_enter(
        &mut self,
        window: &mut DelphosWindowState,
        ctx: PointerEventCtx<'_, Self>,
        event: &sctk::PointerEvent,
    ) {
    }
    fn pointer_leave(
        &mut self,
        window: &mut DelphosWindowState,
        ctx: PointerEventCtx<'_, Self>,
        event: &sctk::PointerEvent,
    ) {
    }
    fn pointer_press(
        &mut self,
        window: &mut DelphosWindowState,
        ctx: PointerEventCtx<'_, Self>,
        event: &sctk::PointerEvent,
        button: u32,
        time: u32,
    ) {
    }
    fn pointer_release(
        &mut self,
        window: &mut DelphosWindowState,
        ctx: PointerEventCtx<'_, Self>,
        event: &sctk::PointerEvent,
        button: u32,
        time: u32,
    ) {
    }
    fn pointer_motion(
        &mut self,
        window: &mut DelphosWindowState,
        ctx: PointerCtx<'_, Self>,
        event: &sctk::PointerEvent,
        time: u32,
    ) {
    }
    fn pointer_axis(
        &mut self,
        window: &mut DelphosWindowState,
        ctx: PointerCtx<'_, Self>,
        event: &sctk::PointerEvent,
        time: u32,
        horizontal: sctk::AxisScroll,
        vertical: sctk::AxisScroll,
        source: Option<wayland::AxisSource>,
    ) {
    }
}

// ------- Draw -------

pub type DrawCtx<'a, State> = boilerplate::CompositorHandlerCtx<'a, DelphosWindow<State>>;

#[expect(unused_variables, reason = "Blank implementations")]
pub trait DelphosWindowDraw: DelphosWindowApp {
    fn draw(&mut self, window: &mut DelphosWindowState, ctx: DrawCtx<'_, Self>, time: u32) {}
}

// ------- Windowing -------

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

            frame_count: 0,
            last_frame_count: 0,
            update_frame_count: 0,
            delta: 0,
            last_delta: 0,
            average_delta: 0,
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
