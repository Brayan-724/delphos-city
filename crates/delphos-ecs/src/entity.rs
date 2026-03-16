use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::{Component, Rwc};

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct EntityId(pub(crate) usize);

#[derive(Default)]
pub struct Entity {
    pub(crate) components: HashMap<TypeId, Rwc<dyn Any>>,
}

impl Entity {
    pub fn component<C: Component>(&self) -> Rwc<C> {
        self.component_checked()
            .expect(&format!("Cannot find component: {}", C::name()))
    }

    pub fn component_checked<C: Component>(&self) -> Option<Rwc<C>> {
        self.components
            .get(&C::id())
            .map(|rwc| rwc.map(|c| c as *mut C))
    }
}
