use impl_variadics::impl_variadics;

use crate::WorldQuery;

impl_variadics! {
    1..10 "T*" => {
        impl<#(#T0: WorldQuery),*> WorldQuery for (#(#T0,)*) {
            fn world_query(world: &mut impl crate::World) -> Option<Self> {
                Some((
                    #(#T0::world_query(world)?,)*
                ))
            }
        }
    }
}
