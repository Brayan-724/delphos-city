use std::marker::PhantomData;

use crate::{Component, ComponentId};

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct EntityId(pub(crate) usize);

impl EntityId {
    pub fn component<C: Component>(self) -> ComponentId<C> {
        ComponentId(self, PhantomData)
    }
}
