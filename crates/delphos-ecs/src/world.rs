use std::any::{Any, TypeId, type_name};
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use crate::{Component, ComponentId, EntityId, Resource, ResourceId, Rwc, UntypedComponentId};

pub trait World: Sized {
    fn insert_resource<R: Resource>(&mut self, resource: R) -> Option<Rwc<R>>;
    fn resource<R: Resource + Default>(&mut self) -> Rwc<R>;

    fn resource_untyped_checked<R: Resource>(&mut self, id: &ResourceId) -> Option<Rwc<R>>;

    fn spawn_component<C: Component>(&mut self, component: C) -> ComponentId<C>;
    fn attach<C: Component>(&mut self, entity: EntityId, component: C) -> ComponentId<C>;

    fn entities<C: Component>(&mut self) -> &HashSet<EntityId>;

    // Panics if component is not found
    fn component_untyped_checked<C: Component>(&self, id: &UntypedComponentId) -> Option<Rwc<C>>;
}

pub trait WorldExt: World {
    fn resource_checked<R: Resource>(&mut self) -> Option<Rwc<R>> {
        self.resource_untyped_checked(&R::RESOURCE_ID)
    }

    // Panics if component is not found
    fn component<C: Component>(&self, id: &ComponentId<C>) -> Rwc<C> {
        self.component_checked(id)
            .expect(&format!("Cannot find component: {}", C::name()))
    }

    /// "Safe" version
    fn component_checked<C: Component>(&self, id: &ComponentId<C>) -> Option<Rwc<C>> {
        self.component_untyped_checked(&id.untyped())
    }
}

impl<W: World> WorldExt for W {}

#[derive(Default)]
pub struct WorldContainer {
    resources: HashMap<ResourceId, Rwc<dyn Any>>,
    entity_id: usize,
    instances: HashMap<UntypedComponentId, Rwc<dyn Any>>,
    components: HashMap<TypeId, HashSet<EntityId>>,
}

impl World for WorldContainer {
    fn insert_resource<R: Resource>(&mut self, resource: R) -> Option<Rwc<R>> {
        let boxed = self
            .resources
            .insert(R::RESOURCE_ID, Rwc::new(Box::from(resource)));

        boxed.map(|rwc| rwc.map(|d| d as *mut R))
    }

    fn resource<R: Resource + Default>(&mut self) -> Rwc<R> {
        let boxed = self
            .resources
            .entry(R::RESOURCE_ID)
            .or_insert_with(|| Rwc::new(Box::from(R::default())));

        boxed.map(|d| d as *mut R)
    }

    fn resource_untyped_checked<R: Resource>(&mut self, id: &ResourceId) -> Option<Rwc<R>> {
        if id.0 != TypeId::of::<R>() {
            log::debug!(
                "Uncompatible id with resource: {:?} != {:?} ({})",
                id.0,
                TypeId::of::<R>(),
                type_name::<R>()
            );
            return None;
        }

        let boxed = self.resources.get(&id)?;
        Some(boxed.map(|d| d as *mut R))
    }

    fn attach<C: Component>(&mut self, entity: EntityId, component: C) -> ComponentId<C> {
        let id = entity.component::<C>();
        let was_attached = self.components.entry(C::id()).or_default().insert(entity);

        if was_attached {
            log::warn!(
                "Tried to attach component ({}), while it was already attached. Use `update` functions if it was intended",
                C::name()
            );
        }

        self.instances
            .insert(id.untyped(), Rwc::new(Box::from(component)));

        id
    }

    fn entities<C: Component>(&mut self) -> &HashSet<EntityId> {
        self.components.entry(C::id()).or_default()
    }

    fn spawn_component<C: Component>(&mut self, component: C) -> ComponentId<C> {
        let id = EntityId(self.entity_id);
        self.entity_id += 1;

        self.components.entry(C::id()).or_default().insert(id);

        let id = ComponentId::<C>(id, PhantomData);

        self.instances
            .insert(id.untyped(), Rwc::new(Box::from(component)));

        id
    }

    fn component_untyped_checked<C: Component>(&self, id: &UntypedComponentId) -> Option<Rwc<C>> {
        if id.1 != TypeId::of::<C>() {
            log::debug!(
                "Uncompatible id with component: {:?} != {:?} ({})",
                id.1,
                TypeId::of::<C>(),
                C::name()
            );
            return None;
        }

        let boxed = self.instances.get(id)?;
        Some(boxed.map(|rwc| rwc as *mut C))
    }
}

// ------ Query ------

pub trait WorldQuery: Sized {
    fn world_query(world: &mut impl World) -> Option<Self>;
}
