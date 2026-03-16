use std::any::{self, TypeId};
use std::marker::PhantomData;

use crate::EntityId;

pub trait Component: Sized + 'static {
    fn id() -> TypeId {
        TypeId::of::<Self>()
    }

    fn name() -> &'static str {
        any::type_name::<Self>()
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct UntypedComponentId(pub(crate) EntityId, pub(crate) TypeId);

pub struct ComponentId<C: Component>(pub(crate) EntityId, pub(crate) PhantomData<C>);

impl<C: Component> Copy for ComponentId<C> {}
impl<C: Component> Clone for ComponentId<C> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}
impl<C: Component> Eq for ComponentId<C> {}
impl<C: Component> PartialEq for ComponentId<C> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<C: Component> Ord for ComponentId<C> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
impl<C: Component> PartialOrd for ComponentId<C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<C: Component> ComponentId<C> {
    pub fn untyped(self) -> UntypedComponentId {
        UntypedComponentId(self.0, C::id())
    }
}
