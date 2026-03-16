use std::num::Saturating;
use std::ops;

use crate::{Float, Number, NumberCast};

pub type UVec2 = U32Vec2;
pub type U32Vec2 = Vec2<u32>;

pub type IVec2 = I32Vec2;
pub type I32Vec2 = Vec2<i32>;

pub type FVec2 = F32Vec2;
pub type F32Vec2 = Vec2<f32>;

#[derive(Clone, Copy, Debug, Default)]
pub struct Vec2<Unit> {
    pub x: Unit,
    pub y: Unit,
}

impl<Unit: Number> Vec2<Unit> {
    pub const ZERO: Self = Vec2 {
        x: Unit::ZERO,
        y: Unit::ZERO,
    };
}

impl<Unit> Vec2<Saturating<Unit>> {
    pub fn unsaturate(self) -> Vec2<Unit> {
        Vec2 {
            x: self.x.0,
            y: self.y.0,
        }
    }
}

impl<Unit> Vec2<Unit> {
    pub fn new(x: Unit, y: Unit) -> Self {
        Self { x, y }
    }

    pub fn splat(n: Unit) -> Self
    where
        Unit: Copy,
    {
        Self { x: n, y: n }
    }

    pub fn set_x(self, x: Unit) -> Self {
        Self { x, ..self }
    }

    pub fn set_y(self, y: Unit) -> Self {
        Self { y, ..self }
    }

    pub fn saturate(self) -> Vec2<Saturating<Unit>> {
        Vec2 {
            x: Saturating(self.x),
            y: Saturating(self.y),
        }
    }

    pub fn round(self) -> Self
    where
        Unit: Float,
    {
        Self {
            x: self.x.round(),
            y: self.y.round(),
        }
    }

    pub fn len(self) -> Unit
    where
        Unit: Float,
    {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}

impl<Unit: Copy> Vec2<Unit> {
    pub fn xx(self) -> Self {
        Self {
            x: self.x,
            y: self.x,
        }
    }

    pub fn yy(self) -> Self {
        Self {
            x: self.x,
            y: self.x,
        }
    }
}

impl<Unit> From<(Unit, Unit)> for Vec2<Unit> {
    fn from((x, y): (Unit, Unit)) -> Self {
        Self::new(x, y)
    }
}

impl<Unit> From<[Unit; 2]> for Vec2<Unit> {
    fn from([x, y]: [Unit; 2]) -> Self {
        Self::new(x, y)
    }
}

macro_rules! impl_cast {
    ($($fn:ident => $ty:ty;)*) => {
        impl<Unit> Vec2<Unit> {
        $(
            pub fn $fn(self) -> Vec2<$ty>
            where
                Unit: Number + NumberCast<$ty>,
            {
                Vec2 {
                    x: self.x.cast(),
                    y: self.y.cast(),
                }
            }
        )*
        }
    };
}

impl_cast!(
    as_u8 => u8; as_u16 => u16; as_u32 => u32; as_u64 => u64;
    as_i8 => i8; as_i16 => i16; as_i32 => i32; as_i64 => i64;
    as_f32 => f32; as_f64 => f64;
);

macro_rules! impl_op {
    (
        [op]
        $($trait:ident, $fn:ident, $op:tt;)*
        [assign]
        $($atrait:ident, $afn:ident, $aop:tt;)*
    ) => {

        impl<Unit: ops::Neg> ops::Neg for Vec2<Unit> {
            type Output = Vec2<Unit::Output>;

            fn neg(self) -> Self::Output {
                Vec2 {
                    x: -self.x,
                    y: -self.y,
                }
            }
        }

        $(
        impl<Unit: ops::$trait> ops::$trait for Vec2<Unit> {
            type Output = Vec2<Unit::Output>;

            fn $fn(self, rhs: Self) -> Self::Output {
                Vec2 {
                    x: self.x $op rhs.x,
                    y: self.y $op rhs.y,
                }
            }
        }

        impl<Unit: ops::$trait> ops::$trait<(Unit, Unit)> for Vec2<Unit> {
            type Output = Vec2<Unit::Output>;

            fn $fn(self, rhs: (Unit, Unit)) -> Self::Output {
                Vec2 {
                    x: self.x $op rhs.0,
                    y: self.y $op rhs.1,
                }
            }
        }

        impl<Unit: ops::$trait + Copy> ops::$trait<Unit> for Vec2<Unit> {
            type Output = Vec2<Unit::Output>;

            fn $fn(self, rhs: Unit) -> Self::Output {
                Vec2 {
                    x: self.x $op rhs,
                    y: self.y $op rhs,
                }
            }
        }
    )*
    $(
        impl<Unit: ops::$atrait> ops::$atrait for Vec2<Unit> {
            fn $afn(&mut self, rhs: Self)  {
                self.x $aop rhs.x;
                self.y $aop rhs.y;
            }
        }

        impl<Unit: ops::$atrait + Copy> ops::$atrait<Unit> for Vec2<Unit> {
            fn $afn(&mut self, rhs: Unit) {
                self.x $aop rhs;
                self.y $aop rhs;
            }
        }
    )*};
}

impl_op!(
    [op]
    Add, add, +;
    Sub, sub, -;
    Mul, mul, *;
    Div, div, /;
    [assign]
    AddAssign, add_assign, +=;
    SubAssign, sub_assign, -=;
    MulAssign, mul_assign, *=;
    DivAssign, div_assign, /=;
);
