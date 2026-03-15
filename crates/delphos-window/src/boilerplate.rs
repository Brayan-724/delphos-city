use smithay_client_toolkit::output::OutputState;
use smithay_client_toolkit::registry::RegistryState;
use smithay_client_toolkit::seat::SeatState;
use smithay_client_toolkit::seat::pointer::{PointerEvent, PointerEventKind};
use smithay_client_toolkit::shell::wlr_layer::LayerSurface;
use smithay_client_toolkit::shm::Shm;
use wayland_client::protocol::wl_keyboard::WlKeyboard;
use wayland_client::protocol::wl_output::WlOutput;
use wayland_client::protocol::wl_pointer::WlPointer;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Connection, QueueHandle};

use crate::macros;

pub struct HandlerCtx<'a, State, Data> {
    pub conn: &'a Connection,
    pub qh: &'a QueueHandle<State>,
    pub data: Data,
}

// --------- Compositor ---------

pub type CompositorHandlerCtx<'a, State> = HandlerCtx<'a, State, &'a WlSurface>;

macros::create_delegate! {[delegate_compositor]
    [::smithay_client_toolkit::compositor::CompositorHandler]
    [crate::boilerplate]
    [CompositorHandler]
    [CompositorHandlerCtx]
    data:
        []
        [surface: &crate::wayland::WlSurface,]
        [surface]
    impl: {
        scale_factor_changed(new_factor: i32);
        transform_changed(new_transform: crate::wayland::Transform);
        frame(time: u32);
        surface_enter(output: &crate::wayland::WlOutput);
        surface_leave(output: &crate::wayland::WlOutput);
    }
    trait: {}
}

// --------- Output ---------

pub type OutputHandlerCtx<'a, State> = HandlerCtx<'a, State, WlOutput>;

macros::create_delegate! {[delegate_output]
    [::smithay_client_toolkit::output::OutputHandler]
    [crate::boilerplate]
    [OutputHandler : BaseHandler]
    [OutputHandlerCtx]
    data:
        []
        [output: crate::wayland::WlOutput,]
        [output]
    impl: {
        new_output();
        update_output();
        output_destroyed();
    }
    impl-extra: {
        fn output_state(&mut self) -> &mut OutputState {
            crate::boilerplate::BaseHandler::output_state(self)
        }
    }
    trait: {
    }
}

// --------- Layer ---------

pub type LayerHandlerCtx<'a, State> = HandlerCtx<'a, State, &'a LayerSurface>;

macros::create_delegate! {[delegate_layer]
    [::smithay_client_toolkit::shell::wlr_layer::LayerShellHandler]
    [crate::boilerplate]
    [LayerHandler]
    [LayerHandlerCtx]
    data:
        []
        [layer: &crate::sctk::LayerSurface,]
        [layer]
    impl: {
        closed();
        configure(configure: crate::sctk::LayerSurfaceConfigure, serial: u32);
    }
    trait: {}
}

// --------- Seat ---------

pub type SeatHandlerCtx<'a, State> = HandlerCtx<'a, State, WlSeat>;

macros::create_delegate! {[delegate_seat]
    [::smithay_client_toolkit::seat::SeatHandler]
    [crate::boilerplate]
    [SeatHandler : BaseHandler]
    [SeatHandlerCtx]
    data:
        []
        [seat: crate::wayland::WlSeat,]
        [seat]
    impl: {
        new_seat();
        remove_seat();
        new_capability(capability: crate::sctk::Capability);
        remove_capability(capability: crate::sctk::Capability);
    }
    impl-extra: {
        fn seat_state(&mut self) -> &mut crate::sctk::SeatState {
            crate::window::boilerplate::BaseHandler::seat_state(self)
        }
    }
    trait: { }
}

// --------- Keyboard ---------

pub type KeyboardHandlerCtx<'a, State> = HandlerCtx<'a, State, (&'a WlKeyboard, u32)>;

macros::create_delegate! {[delegate_keyboard]
    [::smithay_client_toolkit::seat::keyboard::KeyboardHandler]
    [crate::boilerplate]
    [KeyboardHandler]
    [KeyboardHandlerCtx]
    data:
        []
        [keyboard: &crate::wayland::WlKeyboard, serial: u32,]
        [(keyboard, serial)]
    impl: {
        press_key  (event: crate::sctk::KeyEvent);
        repeat_key (event: crate::sctk::KeyEvent);
        release_key(event: crate::sctk::KeyEvent);
        update_modifiers(
            modifiers: crate::sctk::Modifiers,
            raw_modifiers: crate::sctk::RawModifiers,
            layout: u32
        );
    }
    impl-extra: {
        crate::macros::create_delegate! {#impl-fn
            [Self] [KeyboardHandler]
            Use: [
                [use crate::boilerplate::KeyboardHandler;]
                [use crate::boilerplate::KeyboardHandlerCtx;]
            ]
            [KeyboardHandlerCtx]
            [enter]
            Data:
                [keyboard: &crate::wayland::WlKeyboard,]
                [(keyboard, serial)]
            Args:
                [
                    surface: &crate::wayland::WlSurface,
                    serial: u32,
                    raw: &[u32],
                    keysyms: &[crate::sctk::Keysym],
                ]
                [surface, raw, keysyms]
        }
        crate::macros::create_delegate! {#impl-fn
            [Self] [KeyboardHandler]
            Use: [
                [use crate::boilerplate::KeyboardHandler;]
                [use crate::boilerplate::KeyboardHandlerCtx;]
            ]
            [KeyboardHandlerCtx]
            [leave]
            Data:
                [keyboard: &crate::wayland::WlKeyboard,]
                [(keyboard, serial)]
            Args:
                [surface: &crate::wayland::WlSurface, serial: u32,]
                [surface]
        }
    }
    trait: {
        macros::create_delegate! {#trait-fn
             [Self] [KeyboardHandlerCtx]
             [enter]
             Args:
                 [
                    surface: &crate::wayland::WlSurface,
                    raw: &[u32],
                    keysyms: &[crate::sctk::Keysym],
                 ] [[surface] [raw] [keysyms]]
        }
        macros::create_delegate! {#trait-fn
             [Self] [KeyboardHandlerCtx]
             [leave]
             Args: [surface: &crate::wayland::WlSurface,] [[surface]]
        }
    }
}

// --------- Pointer ---------

pub type PointerHandlerCtx<'a, State> = HandlerCtx<'a, State, &'a WlPointer>;
pub type PointerEventHandlerCtx<'a, State> = HandlerCtx<'a, State, (&'a WlPointer, u32)>;

fn ctx_copy<'a, State>(ctx: &PointerHandlerCtx<'a, State>) -> PointerHandlerCtx<'a, State> {
    PointerHandlerCtx {
        conn: ctx.conn,
        qh: ctx.qh,
        data: ctx.data,
    }
}

fn pointer_ctx<'a, State>(
    ctx: &PointerHandlerCtx<'a, State>,
    serial: u32,
) -> PointerEventHandlerCtx<'a, State> {
    PointerEventHandlerCtx {
        conn: ctx.conn,
        qh: ctx.qh,
        data: (ctx.data, serial),
    }
}

pub trait PointerHandler: Sized {
    fn pointer_frame(&mut self, ctx: PointerHandlerCtx<'_, Self>, events: &[PointerEvent]) {
        use PointerEventKind::*;
        for event in events {
            match event.kind {
                Enter { serial, .. } => self.pointer_enter(pointer_ctx(&ctx, serial), event),
                Leave { serial, .. } => self.pointer_leave(pointer_ctx(&ctx, serial), event),
                Motion { .. } => self.pointer_motion(ctx_copy(&ctx), event),
                Press { serial, .. } => self.pointer_press(pointer_ctx(&ctx, serial), event),
                Release { serial, .. } => self.pointer_release(pointer_ctx(&ctx, serial), event),
                Axis { .. } => self.pointer_axis(ctx_copy(&ctx), event),
            }
        }
    }

    fn pointer_enter(&mut self, ctx: PointerEventHandlerCtx<'_, Self>, event: &PointerEvent) {
        _ = ctx;
        _ = event;
    }

    fn pointer_leave(&mut self, ctx: PointerEventHandlerCtx<'_, Self>, event: &PointerEvent) {
        _ = ctx;
        _ = event;
    }

    fn pointer_motion(&mut self, ctx: PointerHandlerCtx<'_, Self>, event: &PointerEvent) {
        _ = ctx;
        _ = event;
    }

    fn pointer_press(&mut self, ctx: PointerEventHandlerCtx<'_, Self>, event: &PointerEvent) {
        _ = ctx;
        _ = event;
    }

    fn pointer_release(&mut self, ctx: PointerEventHandlerCtx<'_, Self>, event: &PointerEvent) {
        _ = ctx;
        _ = event;
    }

    fn pointer_axis(&mut self, ctx: PointerHandlerCtx<'_, Self>, event: &PointerEvent) {
        _ = ctx;
        _ = event;
    }
}

macro_rules! delegate_pointer {
    (impl [$($gen:tt)*] for $State:ty) => {
        ::smithay_client_toolkit::delegate_pointer!(@$($gen)* $State);
        impl $($gen)* ::smithay_client_toolkit::seat::pointer::PointerHandler for $State {
            fn pointer_frame(
                &mut self,
                conn: &::wayland_client::Connection,
                qh: &::wayland_client::QueueHandle<Self>,
                pointer: &::wayland_client::protocol::wl_pointer::WlPointer,
                events: &[::smithay_client_toolkit::seat::pointer::PointerEvent],
            ) {
                use crate::window::boilerplate::PointerHandler;
                use crate::window::boilerplate::PointerHandlerCtx;

                <Self as PointerHandler>::pointer_frame(
                    self,
                    PointerHandlerCtx {
                        conn,
                        qh,
                        data: pointer,
                    },
                    events,
                );
            }
        }
    };
}

// --------- Commmon ---------

pub trait BaseHandler {
    fn registry(&mut self) -> &mut RegistryState;
    fn shm_state(&mut self) -> &mut Shm;
    fn output_state(&mut self) -> &mut OutputState;
    fn seat_state(&mut self) -> &mut SeatState;
}

macro_rules! delegate_base {
    (impl [$($gen:tt)*] for $State:ty) => {
        ::smithay_client_toolkit::delegate_shm!(@$($gen)* $State);
        impl $($gen)* ::smithay_client_toolkit::shm::ShmHandler for $State {
            fn shm_state(&mut self) -> &mut ::smithay_client_toolkit::shm::Shm {
                <Self as $crate::boilerplate::BaseHandler>::shm_state(self)
            }
        }

        ::smithay_client_toolkit::delegate_registry!(@$($gen)* $State);
        impl $($gen)* ::smithay_client_toolkit::registry::ProvidesRegistryState for $State {
            fn registry(&mut self) -> &mut ::smithay_client_toolkit::registry::RegistryState {
                <Self as $crate::boilerplate::BaseHandler>::registry(self)
            }
            ::smithay_client_toolkit::registry_handlers![OutputState, SeatState];
        }
    };
}

pub(crate) use delegate_base;
pub(crate) use delegate_compositor;
pub(crate) use delegate_keyboard;
pub(crate) use delegate_layer;
pub(crate) use delegate_output;
pub(crate) use delegate_pointer;
pub(crate) use delegate_seat;
