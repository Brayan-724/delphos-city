use std::fmt;

use delphos_math::IVec2;

use crate::{DelphosWindow, DelphosWorld, OpenWindow, boilerplate, sctk, wayland};

pub type ConfigureCtx<'a, State> = boilerplate::LayerHandlerCtx<'a, DelphosWindow<State>>;

pub trait DelphosWindowApp: Sized + 'static {
    fn setup(pos: IVec2, size: IVec2, output: wayland::WlOutput) -> OpenWindow;

    type NewError: fmt::Debug;
    fn new(world: &mut DelphosWorld) -> Result<Self, Self::NewError>;

    fn configure(
        &mut self,
        world: &mut DelphosWorld,
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
        world: &mut DelphosWorld,
        ctx: KeyboardCtx<'_, Self>,
        event: sctk::KeyEvent,
    ) {
    }
    fn repeat_key(
        &mut self,
        world: &mut DelphosWorld,
        ctx: KeyboardCtx<'_, Self>,
        event: sctk::KeyEvent,
    ) {
    }
    fn release_key(
        &mut self,
        world: &mut DelphosWorld,
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
        world: &mut DelphosWorld,
        ctx: PointerEventCtx<'_, Self>,
        event: &sctk::PointerEvent,
    ) {
    }
    fn pointer_leave(
        &mut self,
        world: &mut DelphosWorld,
        ctx: PointerEventCtx<'_, Self>,
        event: &sctk::PointerEvent,
    ) {
    }
    fn pointer_press(
        &mut self,
        world: &mut DelphosWorld,
        ctx: PointerEventCtx<'_, Self>,
        event: &sctk::PointerEvent,
        button: u32,
        time: u32,
    ) {
    }
    fn pointer_release(
        &mut self,
        world: &mut DelphosWorld,
        ctx: PointerEventCtx<'_, Self>,
        event: &sctk::PointerEvent,
        button: u32,
        time: u32,
    ) {
    }
    fn pointer_motion(
        &mut self,
        world: &mut DelphosWorld,
        ctx: PointerCtx<'_, Self>,
        event: &sctk::PointerEvent,
        time: u32,
    ) {
    }
    fn pointer_axis(
        &mut self,
        world: &mut DelphosWorld,
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
    fn draw(&mut self, world: &mut DelphosWorld, ctx: DrawCtx<'_, Self>) {}
}
