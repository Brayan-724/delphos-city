use std::ops;

use crate::macros;

macro_rules! mark {
    ($trait:ident: $($ty:ty),*) => {
        $(impl $trait for $ty {})*
    };
}

pub trait Number:
    ops::Add<Output = Self>
    + ops::Sub<Output = Self>
    + ops::Mul<Output = Self>
    + ops::Div<Output = Self>
    + Sized
    + Copy
    + Clone
{
    fn cast<T>(self) -> T
    where
        Self: NumberCast<T>,
    {
        self.number_cast()
    }
}
mark!(Number: u8, u16, u32, u64, u128);
mark!(Number: i8, i16, i32, i64, i128);
mark!(Number: f32, f64);

pub trait Unsigned: Number {}
mark!(Unsigned: u8, u16, u32, u64, u128);

pub trait Integer: Number {}
mark!(Integer: i8, i16, i32, i64, i128);

pub trait Float: Number {
    fn powi(self, rhs: i32) -> Self;
    fn round(self) -> Self;
    fn sqrt(self) -> Self;
}
macro_rules! mark_float {
    ($($ty:ty),*) => {
        $(impl Float for $ty {
            fn powi(self, rhs: i32) -> Self {
                self.powi(rhs)
            }
            fn round(self) -> Self {
                self.round()
            }
            fn sqrt(self) -> Self {
                self.sqrt()
            }
        })*
    };
}
mark_float!(f32, f64);

pub trait NumberCast<T> {
    fn number_cast(self) -> T;
}

macro_rules! impl_cast {
    ($a:tt, $b:tt) => {
        impl_cast!(@ $a, $b);
        impl_cast!(@ $b, $a);
    };

    (@ (u, $a:ty), (i, $b:ty)) => {
        impl NumberCast<$b> for $a {
            fn number_cast(self) -> $b {
                self.cast_signed() as $b
            }
        }
    };

    (@ (i, $a:ty), (u, $b:ty)) => {
        impl NumberCast<$b> for $a {
            fn number_cast(self) -> $b {
                self.max(0).unsigned_abs() as $b
            }
        }
    };

    (@ (f, $a:ty), (f, $b:ty)) => {
        impl NumberCast<$b> for $a {
            fn number_cast(self) -> $b {
                self as $b
            }
        }
    };

    (@ (f, $a:ty), ($_:ident, $b:ty)) => {
        impl NumberCast<$b> for $a {
            fn number_cast(self) -> $b {
                self.trunc() as $b
            }
        }
    };

    (@ ($_:ident, $a:ty), ($__:ident, $b:ty)) => {
        impl NumberCast<$b> for $a {
            fn number_cast(self) -> $b {
                self as $b
            }
        }
    };
}

#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
const _: () = {
    #[rustfmt::skip]
    macros::dual_combination!(impl_cast, [
        (u, usize), (u, u8), (u, u16), (u, u32), (u, u64),
        (i, isize), (i, i8), (i, i16), (i, i32), (i, i64),
        (f, f32  ), (f, f64),
    ]);
};
