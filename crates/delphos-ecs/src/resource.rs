use std::any;

pub trait Resource: Default + Sized + 'static {
    fn id() -> any::TypeId {
        any::TypeId::of::<Self>()
    }

    fn name() -> &'static str {
        any::type_name::<Self>()
    }
}
