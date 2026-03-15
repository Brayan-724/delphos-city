
pub use app::*;
pub use open::*;

mod app;
mod boilerplate;
mod macros;
mod open;
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
