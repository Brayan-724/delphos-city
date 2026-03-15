use smithay_client_toolkit::output::OutputState;
use smithay_client_toolkit::registry::RegistryState;
use smithay_client_toolkit::seat::{
    Capability, SeatState,
    pointer::{PointerEvent, PointerEventKind},
};
use smithay_client_toolkit::shell::WaylandSurface;
use smithay_client_toolkit::shm::Shm;

use crate::boilerplate::{
    self, BaseHandler, CompositorHandler, KeyboardHandler, LayerHandler, OutputHandler,
    PointerHandler, SeatHandler,
};
use crate::{
    DelphosWindow, DelphosWindowApp, DelphosWindowDraw, DelphosWindowKeyboard, DelphosWindowPointer,
};

impl<State: DelphosWindowKeyboard> KeyboardHandler for DelphosWindow<State> {
    fn press_key(
        &mut self,
        ctx: boilerplate::KeyboardHandlerCtx<'_, Self>,
        event: crate::sctk::KeyEvent,
    ) {
        log::debug!(target: "keyboard", "Pressed: {event:?}");
        self.app.press_key(&mut self.window, ctx, event);
    }

    fn repeat_key(
        &mut self,
        ctx: boilerplate::KeyboardHandlerCtx<'_, Self>,
        event: crate::sctk::KeyEvent,
    ) {
        self.app.repeat_key(&mut self.window, ctx, event);
    }

    fn release_key(
        &mut self,
        ctx: boilerplate::KeyboardHandlerCtx<'_, Self>,
        event: crate::sctk::KeyEvent,
    ) {
        log::debug!(target: "keyboard", "Released: {event:?}");
        self.app.release_key(&mut self.window, ctx, event);
    }
}

impl<State: DelphosWindowPointer> PointerHandler for DelphosWindow<State> {
    fn pointer_enter(
        &mut self,
        ctx: boilerplate::PointerEventHandlerCtx<'_, Self>,
        event: &PointerEvent,
    ) {
        let PointerEventKind::Enter { .. } = event.kind else {
            return;
        };
        log::debug!(target: "pointer", "Enter: {:?}", event.position);
        self.app.pointer_enter(&mut self.window, ctx, event);
    }

    fn pointer_leave(
        &mut self,
        ctx: boilerplate::PointerEventHandlerCtx<'_, Self>,
        event: &PointerEvent,
    ) {
        let PointerEventKind::Leave { .. } = event.kind else {
            return;
        };
        log::debug!(target: "pointer", "Leave: {:?}", event.position);
        self.app.pointer_leave(&mut self.window, ctx, event);
    }

    fn pointer_press(
        &mut self,
        ctx: boilerplate::PointerEventHandlerCtx<'_, Self>,
        event: &PointerEvent,
    ) {
        let PointerEventKind::Press { button, time, .. } = event.kind else {
            return;
        };
        log::debug!(target: "pointer", "Press ({}): {:?}", button, event.position);
        self.app
            .pointer_press(&mut self.window, ctx, event, button, time);
    }

    fn pointer_release(
        &mut self,
        ctx: boilerplate::PointerEventHandlerCtx<'_, Self>,
        event: &PointerEvent,
    ) {
        let PointerEventKind::Release { button, time, .. } = event.kind else {
            return;
        };
        log::debug!(target: "pointer", "Release ({}): {:?}", button, event.position);
        self.app
            .pointer_release(&mut self.window, ctx, event, button, time);
    }

    fn pointer_motion(
        &mut self,
        ctx: boilerplate::PointerHandlerCtx<'_, Self>,
        event: &PointerEvent,
    ) {
        let PointerEventKind::Motion { time } = event.kind else {
            return;
        };
        log::debug!(target: "pointer", "Motion ({}): {:?}", time, event.position);
        self.app.pointer_motion(&mut self.window, ctx, event, time);
    }

    fn pointer_axis(
        &mut self,
        ctx: boilerplate::PointerHandlerCtx<'_, Self>,
        event: &PointerEvent,
    ) {
        let PointerEventKind::Axis {
            time,
            horizontal,
            vertical,
            source,
        } = event.kind
        else {
            return;
        };
        log::debug!(target: "pointer", "Scroll {event:?}");
        self.app.pointer_axis(
            &mut self.window,
            ctx,
            event,
            time,
            horizontal,
            vertical,
            source,
        );
    }
}

impl<State: DelphosWindowDraw> CompositorHandler for DelphosWindow<State> {
    fn frame(&mut self, ctx: boilerplate::CompositorHandlerCtx<'_, Self>, time: u32) {
        if self.window.last_delta != 0 {
            self.window.delta = time.abs_diff(self.window.last_delta);
            self.window.average_delta += self.window.delta;
        }
        self.window.last_delta = time;

        self.window.frame_count += 1;

        if time > self.window.update_frame_count {
            if self.window.update_frame_count != 0 {
                let frames = self.window.frame_count - self.window.last_frame_count;
                self.window.last_frame_count = self.window.frame_count;

                let average_delta = self.window.average_delta / frames;
                log::debug!(target: "window::stats", "FPS: {frames} @ {average_delta}ms");

                self.window.average_delta = 0;
            }

            self.window.update_frame_count = time + 1000;
        }

        self.app.draw(&mut self.window, ctx, time);
    }
}

impl<State: DelphosWindowApp> BaseHandler for DelphosWindow<State> {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.window.registry
    }

    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.window.seat
    }

    fn output_state(&mut self) -> &mut OutputState {
        &mut self.window.output
    }

    fn shm_state(&mut self) -> &mut Shm {
        &mut self.window.shm
    }
}

impl<State: DelphosWindowDraw> LayerHandler for DelphosWindow<State> {
    fn configure(
        &mut self,
        ctx: boilerplate::LayerHandlerCtx<'_, Self>,
        configure: crate::sctk::LayerSurfaceConfigure,
        serial: u32,
    ) {
        self.app
            .configure(&mut self.window, &ctx, configure, serial);

        if !self.window.configured {
            self.window.configured = true;

            let ctx = super::DrawCtx {
                conn: ctx.conn,
                qh: ctx.qh,
                data: ctx.data.wl_surface(),
            };

            self.app.draw(&mut self.window, ctx, 0);
        }
    }
}

impl<State: DelphosWindowApp> OutputHandler for DelphosWindow<State> {}

impl<State: DelphosWindowKeyboard + DelphosWindowPointer> SeatHandler for DelphosWindow<State> {
    fn new_capability(
        &mut self,
        ctx: boilerplate::SeatHandlerCtx<'_, Self>,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard {
            log::debug!(target: "window::seat", "Enabled keyboard capability");
            _ = self.window.seat.get_keyboard(ctx.qh, &ctx.data, None);
        }

        if capability == Capability::Pointer {
            log::debug!(target: "window::seat", "Enabled pointer capability");
            _ = self.window.seat.get_pointer(ctx.qh, &ctx.data);
        }
    }
}

boilerplate::delegate_keyboard!  (impl [<State: DelphosWindowKeyboard>] for DelphosWindow<State>);
boilerplate::delegate_pointer!   (impl [<State: DelphosWindowPointer>] for DelphosWindow<State>);
boilerplate::delegate_compositor!(impl [<State: DelphosWindowDraw>] for DelphosWindow<State>);
boilerplate::delegate_base!      (impl [<State: DelphosWindowDraw + DelphosWindowKeyboard + DelphosWindowPointer>] for DelphosWindow<State>);
boilerplate::delegate_layer!     (impl [<State: DelphosWindowDraw>] for DelphosWindow<State>);
boilerplate::delegate_output!    (impl [<State: DelphosWindowApp>] for DelphosWindow<State>);
boilerplate::delegate_seat!      (impl [<State: DelphosWindowKeyboard + DelphosWindowPointer>] for DelphosWindow<State>);
