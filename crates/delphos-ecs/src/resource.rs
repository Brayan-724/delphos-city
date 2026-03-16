use std::any;

use crate::{Rwc, World};

pub trait Resource: Default + Sized + 'static {
    fn id() -> any::TypeId {
        any::TypeId::of::<Self>()
    }

    fn name() -> &'static str {
        any::type_name::<Self>()
    }

    fn get(world: &mut impl World) -> Rwc<Self> {
        world.resource::<Self>()
    }
}
