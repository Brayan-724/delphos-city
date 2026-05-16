use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::{any, ops};

use crate::{Rwc, World, WorldExt, WorldQuery};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct UntypedResourceId(pub(crate) TypeId, pub(crate) TypeId);

impl UntypedResourceId {
    pub const fn new<Base: Resource, Diff: Any + ?Sized>() -> Self {
        Self(TypeId::of::<Base>(), TypeId::of::<Diff>())
    }

    pub fn get<R: Resource>(&self, world: &mut impl World) -> Rwc<R> {
        world
            .resource_untyped_checked::<R>(self)
            .expect(&format!("Cannot find resource: {}", R::name()))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId<R: Resource>(pub(crate) PhantomData<R>, pub(crate) TypeId);

impl<R: Resource> ResourceId<R> {
    pub const fn new<Diff: Any + ?Sized>() -> Self {
        Self(PhantomData, TypeId::of::<Diff>())
    }

    pub fn untyped(&self) -> UntypedResourceId {
        UntypedResourceId(TypeId::of::<R>(), self.1)
    }

    pub fn get(&self, world: &mut impl World) -> Rwc<R> {
        world
            .resource_untyped_checked::<R>(&self.untyped())
            .expect(&format!("Cannot find resource: {}", R::name()))
    }
}

pub trait Resource: Any + Sized {
    const RESOURCE_ID: UntypedResourceId = UntypedResourceId::new::<Self, ()>();

    fn name() -> &'static str {
        any::type_name::<Self>()
    }

    fn get(world: &mut impl World) -> Rwc<Self>
    where
        Self: Default,
    {
        world.resource::<Self>()
    }
}

/// Search for a [`Resource`] in active [`World`],
/// if `CHECKED` is `true` then
pub struct Res<R: Resource, const CHECKED: bool = false>(pub Rwc<R>);

impl<R: Resource, const CHECKED: bool> ops::Deref for Res<R, CHECKED> {
    type Target = Rwc<R>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<R: Resource + Default> WorldQuery for Res<R, false> {
    fn world_query(world: &mut impl World) -> Option<Self> {
        Some(Res(world.resource::<R>()))
    }
}

impl<R: Resource> WorldQuery for Res<R, true> {
    fn world_query(world: &mut impl World) -> Option<Self> {
        world.resource_checked::<R>().map(Res)
    }
}
