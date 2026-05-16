use delphos_ecs::{World, WorldContainer};

pub use app::*;
pub use open::*;
pub use resources::*;

mod app;
mod boilerplate;
mod macros;
mod open;
mod outputs;
mod resources;
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
    pub world: DelphosWorld,
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

    configured: bool,
}

pub struct DelphosWorld {
    pub window: DelphosWindowState,
    container: WorldContainer,

    exit: bool,
}

impl DelphosWorld {
    pub fn exit(&mut self) {
        self.exit = true;
    }

    pub fn window(&self) -> &DelphosWindowState {
        &self.window
    }

    pub fn window_mut(&mut self) -> &mut DelphosWindowState {
        &mut self.window
    }
}

impl World for DelphosWorld {
    fn insert_resource<R: delphos_ecs::Resource>(
        &mut self,
        resource: R,
    ) -> Option<delphos_ecs::Rwc<R>> {
        self.container.insert_resource(resource)
    }

    fn resource<R: delphos_ecs::Resource + Default>(&mut self) -> delphos_ecs::Rwc<R> {
        self.container.resource()
    }

    fn resource_checked<R: delphos_ecs::Resource>(&mut self) -> Option<delphos_ecs::Rwc<R>> {
        self.container.resource_checked()
    }

    fn entities<C: delphos_ecs::Component>(
        &mut self,
    ) -> &std::collections::HashSet<delphos_ecs::EntityId> {
        self.container.entities::<C>()
    }

    fn attach<C: delphos_ecs::Component>(
        &mut self,
        entity: delphos_ecs::EntityId,
        component: C,
    ) -> delphos_ecs::ComponentId<C> {
        self.container.attach(entity, component)
    }

    fn spawn_component<C: delphos_ecs::Component>(
        &mut self,
        component: C,
    ) -> delphos_ecs::ComponentId<C> {
        self.container.spawn_component(component)
    }

    fn component<C: delphos_ecs::Component>(
        &self,
        id: &delphos_ecs::ComponentId<C>,
    ) -> delphos_ecs::Rwc<C> {
        self.container.component(id)
    }

    fn component_checked<C: delphos_ecs::Component>(
        &self,
        id: &delphos_ecs::ComponentId<C>,
    ) -> Option<delphos_ecs::Rwc<C>> {
        self.container.component_checked(id)
    }
}
