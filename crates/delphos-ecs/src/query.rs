use crate::{Component, Rwc, WorldExt, WorldQuery};

pub struct First<C: Component>(pub Rwc<C>);

impl<C: Component> WorldQuery for First<C> {
    fn world_query(world: &mut impl crate::World) -> Option<Self> {
        let entities = world.entities::<C>();
        let entity = *entities.iter().next()?;

        world.component_checked(&entity.component::<C>()).map(First)
    }
}

pub struct Query<C: Component>(pub Vec<Rwc<C>>);

impl<C: Component> WorldQuery for Query<C> {
    fn world_query(world: &mut impl crate::World) -> Option<Self> {
        let entities = world.entities::<C>();
        let components = entities
            .iter()
            .map(|e| e.component::<C>())
            .collect::<Vec<_>>();

        let mut query = Query(Vec::with_capacity(components.len()));

        for id in components {
            query.0.push(world.component(&id));
        }

        Some(query)
    }
}
