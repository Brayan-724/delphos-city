use std::any::{Any, TypeId, type_name};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{Component, ComponentId, Entity, EntityId, Resource, UntypedComponentId};

pub trait World {
    fn insert_resource<R: Resource>(&mut self, resource: R) -> Option<Rwc<R>>;
    fn resource<R: Resource>(&mut self) -> Rwc<R>;

    fn spawn(&mut self) -> Rwc<Entity>;

    fn entity(&self, id: &EntityId) -> Rwc<Entity>;
    fn entity_checked(&self, id: &EntityId) -> Option<Rwc<Entity>>;

    fn spawn_component<C: Component>(&mut self, component: C) -> ComponentId<C>;

    // Panics if component is not found
    fn component<C: Component>(&self, id: &ComponentId<C>) -> Rwc<C>;

    /// "Safe" version
    fn component_checked<C: Component>(&self, id: &ComponentId<C>) -> Option<Rwc<C>>;
}

#[derive(Default)]
pub struct WorldContainer {
    resources: HashMap<TypeId, Rwc<dyn Any>>,
    entity_id: usize,
    entities: HashMap<EntityId, Rwc<Entity>>,
    components: HashMap<UntypedComponentId, Rwc<dyn Any>>,
}

impl World for WorldContainer {
    fn insert_resource<R: Resource>(&mut self, resource: R) -> Option<Rwc<R>> {
        let boxed = self
            .resources
            .insert(R::id(), Rwc::new(Box::from(resource)));

        boxed.map(|rwc| rwc.map(|d| d as *mut R))
    }

    fn resource<R: Resource>(&mut self) -> Rwc<R> {
        let boxed = self
            .resources
            .entry(R::id())
            .or_insert_with(|| Rwc::new(Box::from(R::default())));

        boxed.map(|d| d as *mut R)
    }

    fn spawn(&mut self) -> Rwc<Entity> {
        let id = EntityId(self.entity_id);
        self.entity_id += 1;

        self.entities
            .entry(id)
            .insert_entry(Rwc::new(Box::new(Entity::default())))
            .get()
            .clone()
    }

    fn entity(&self, id: &EntityId) -> Rwc<Entity> {
        self.entity_checked(id).expect("Cannot find entity")
    }

    fn entity_checked(&self, id: &EntityId) -> Option<Rwc<Entity>> {
        self.entities.get(&id).cloned()
    }

    fn spawn_component<C: Component>(&mut self, component: C) -> ComponentId<C> {
        let id = EntityId(self.entity_id);
        self.entity_id += 1;

        let id = ComponentId::<C>(id, PhantomData);

        self.components
            .insert(id.untyped(), Rwc::new(Box::from(component)));

        id
    }

    fn component<C: Component>(&self, id: &ComponentId<C>) -> Rwc<C> {
        self.component_checked(id)
            .expect(&format!("Cannot find component: {}", C::name()))
    }

    fn component_checked<C: Component>(&self, id: &ComponentId<C>) -> Option<Rwc<C>> {
        if let Some(c) = self.components.get(&id.untyped()) {
            Some(c.map(|rwc| rwc as *mut C))
        } else {
            self.entities.get(&id.0)?.read().component_checked::<C>()
        }
    }
}

// ------ Read/Write Counter ------

pub struct Rwc<D: ?Sized> {
    writers: Rc<AtomicUsize>,
    readers: Rc<AtomicUsize>,
    data: *mut D,
}

impl<D: ?Sized> Clone for Rwc<D> {
    fn clone(&self) -> Self {
        Self {
            writers: self.writers.clone(),
            readers: self.readers.clone(),
            data: self.data,
        }
    }
}

impl<D: ?Sized> Rwc<D> {
    pub fn new(data: Box<D>) -> Self {
        Self {
            writers: Rc::new(AtomicUsize::new(0)),
            readers: Rc::new(AtomicUsize::new(0)),
            data: Box::leak(data),
        }
    }

    pub fn map<R: ?Sized>(&self, f: impl FnOnce(*mut D) -> *mut R) -> Rwc<R> {
        Rwc {
            writers: self.writers.clone(),
            readers: self.readers.clone(),
            data: f(self.data),
        }
    }
}

impl<D: Sized + 'static> Rwc<D> {
    pub fn read(&self) -> RwcReaderGuard<D> {
        self.readers.fetch_add(1, Ordering::SeqCst);

        if let n @ 1.. = self.writers.load(Ordering::Relaxed) {
            log::error!(target: "ecs::rwc", "A reader was created while {n} writers are alive for {}", type_name::<D>());
        }

        RwcReaderGuard {
            readers: self.readers.clone(),
            inner: self.data,
        }
    }

    pub fn write(&self) -> RwcWriterGuard<D> {
        if let n @ 1.. = self.writers.fetch_add(1, Ordering::SeqCst) {
            log::error!(target: "ecs::rwc", "A writer was created while other {n} writers are alive for {}", type_name::<D>());
        }

        RwcWriterGuard {
            writers: self.writers.clone(),
            inner: self.data,
        }
    }
}

// ------ Reader ------

pub struct RwcReaderGuard<D: ?Sized> {
    readers: Rc<AtomicUsize>,
    inner: *mut D,
}

impl<D: ?Sized> Drop for RwcReaderGuard<D> {
    fn drop(&mut self) {
        self.readers.fetch_sub(1, Ordering::AcqRel);
    }
}

impl<D: ?Sized> ops::Deref for RwcReaderGuard<D> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner }
    }
}

// ------ Writer ------

pub struct RwcWriterGuard<D: ?Sized> {
    writers: Rc<AtomicUsize>,
    inner: *mut D,
}

impl<D: ?Sized> Drop for RwcWriterGuard<D> {
    fn drop(&mut self) {
        self.writers.fetch_sub(1, Ordering::AcqRel);
    }
}

impl<D: ?Sized> ops::Deref for RwcWriterGuard<D> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner }
    }
}

impl<D: ?Sized> ops::DerefMut for RwcWriterGuard<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner }
    }
}
