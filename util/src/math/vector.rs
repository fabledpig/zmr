use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Div;
use std::ops::DivAssign;
use std::ops::Mul;
use std::ops::MulAssign;
use std::ops::Neg;
use std::ops::Sub;
use std::ops::SubAssign;

pub trait VectorType:
    Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
    + Copy
    + 'static
{
}

impl VectorType for i8 {}
impl VectorType for i16 {}
impl VectorType for i32 {}
impl VectorType for i64 {}
impl VectorType for i128 {}
impl VectorType for isize {}
impl VectorType for f32 {}
impl VectorType for f64 {}

pub trait Dot<Rhs = Self> {
    type Output;

    fn dot(self, rhs: Rhs) -> Self::Output;
}

pub trait Cross<Rhs = Self> {
    type Output;

    fn cross(self, rhs: Rhs) -> Self::Output;
}

pub trait CrossAssign<Rhs = Self> {
    fn cross_assign(&mut self, rhs: Rhs);
}

macro_rules! strip_plus {
    (+ $($rest:tt)*) => {
        $($rest)*
    }
}

macro_rules! forward_ref_binop {
    ($name: ident$(<$type: ident>)?,
    $op_name: ident,
    $fn: ident,
    $rhs: ident$(<$rhs_type: ident>)?,
    $output: ident$(<$output_type: ident>)?
) => {
        impl$(<$type: VectorType>)? $op_name<&$rhs$(<$rhs_type>)?> for $name$(<$type>)? {
            type Output = $output$(<$output_type>)?;

            fn $fn(self, rhs: &$rhs$(<$rhs_type>)?) -> Self::Output {
                $op_name::<&$rhs$(<$rhs_type>)?>::$fn(&self, rhs)
            }
        }

        impl$(<$type: VectorType>)? $op_name<$rhs$(<$rhs_type>)?> for &$name$(<$type>)? {
            type Output = $output$(<$output_type>)?;

            fn $fn(self, rhs: $rhs$(<$rhs_type>)?) -> Self::Output {
                $op_name::<&$rhs$(<$rhs_type>)?>::$fn(self, &rhs)
            }
        }

        impl$(<$type: VectorType>)? $op_name<$rhs$(<$rhs_type>)?> for $name$(<$type>)? {
            type Output = $output$(<$output_type>)?;

            fn $fn(self, rhs: $rhs$(<$rhs_type>)?) -> Self::Output {
                $op_name::<&$rhs$(<$rhs_type>)?>::$fn(&self, &rhs)
            }
        }
    };
}

macro_rules! forward_ref_binop_assign {
    (
        $name: ident$(<$type: ident>)?,
        $op_name: ident,
        $fn: ident,
        $assign_op_name: ident,
        $assign_fn: ident,
        $rhs: ident$(<$rhs_type: ident>)?
    ) => {
        impl$(<$type: VectorType>)? $assign_op_name<$rhs$(<$rhs_type>)?> for $name$(<$type>)? {
            fn $assign_fn(&mut self, rhs: $rhs$(<$rhs_type>)?) {
                *self = $op_name::<$rhs$(<$rhs_type>)?>::$fn(*self, rhs);
            }
        }

        impl$(<$type: VectorType>)? $assign_op_name<&$rhs$(<$rhs_type>)?> for $name$(<$type>)? {
            fn $assign_fn(&mut self, rhs: &$rhs$(<$rhs_type>)?) {
                *self = $op_name::<&$rhs$(<$rhs_type>)?>::$fn(*self, rhs);
            }
        }
    };
}

macro_rules! define_vector {
    ($name:ident $(, $component:ident)+) => {
        #[derive(Clone, Copy, PartialEq, Debug)]
        pub struct $name<T> {
            $(pub $component: T),*
        }

        impl<T> $name<T> {
            pub fn new($($component: T,)*) -> Self {
                Self { $($component,)* }
            }
        }

        impl<T: Default> Default for $name<T> {
            fn default() -> Self {
                Self {
                    $($component: T::default(),)*
                }
            }
        }

        impl<T: VectorType> Dot<&$name<T>> for &$name<T> {
            type Output = T;

            fn dot(self, rhs: &$name<T>) -> Self::Output {
                strip_plus!($(+ self.$component * rhs.$component)*)
            }
        }

        forward_ref_binop!($name<T>, Dot, dot, $name<T>, T);

        impl<T: VectorType> Add<&$name<T>> for &$name<T> {
            type Output = $name<T>;

            fn add(self, rhs: &$name<T>) -> Self::Output {
                $name::<T>::new($(self.$component + rhs.$component),*)
            }
        }

        forward_ref_binop!($name<T>, Add, add, $name<T>, $name<T>);
        forward_ref_binop_assign!($name<T>, Add, add, AddAssign, add_assign, $name<T>);

        impl<T: VectorType> Sub<&$name<T>> for &$name<T> {
            type Output = $name<T>;

            fn sub(self, rhs: &$name<T>) -> Self::Output {
                self + (-rhs)
            }
        }

        forward_ref_binop!($name<T>, Sub, sub, $name<T>, $name<T>);
        forward_ref_binop_assign!($name<T>, Sub, sub, SubAssign, sub_assign, $name<T>);

        impl<T: VectorType> Neg for &$name<T> {
            type Output = $name<T>;

            fn neg(self) -> Self::Output {
                $name::<T>::new($(-self.$component),*)
            }
        }

        impl<T: VectorType> Neg for $name<T> {
            type Output = $name<T>;

            fn neg(self) -> Self::Output {
                -&self
            }
        }

        impl<T: VectorType> Mul<&$name<T>> for &$name<T> {
            type Output = $name<T>;

            fn mul(self, rhs: &$name<T>) -> Self::Output {
                $name::<T>::new($(self.$component * rhs.$component),*)
            }
        }

        forward_ref_binop!($name<T>, Mul, mul, $name<T>, $name<T>);
        forward_ref_binop_assign!($name<T>, Mul, mul, MulAssign, mul_assign, $name<T>);

        impl<T: VectorType> Mul<&T> for &$name<T> {
            type Output = $name<T>;

            fn mul(self, rhs: &T) -> Self::Output {
                $name::<T>::new($(self.$component * *rhs),*)
            }
        }

        forward_ref_binop!($name<T>, Mul, mul, T, $name<T>);
        forward_ref_binop_assign!($name<T>, Mul, mul, MulAssign, mul_assign, T);

        impl<T: VectorType> Div<&$name<T>> for &$name<T> {
            type Output = $name<T>;

            fn div(self, rhs: &$name<T>) -> Self::Output {
                $name::<T>::new($(self.$component / rhs.$component),*)
            }
        }

        forward_ref_binop!($name<T>, Div, div, $name<T>, $name<T>);
        forward_ref_binop_assign!($name<T>, Div, div, DivAssign, div_assign, $name<T>);

        impl<T: VectorType> Div<&T> for &$name<T> {
            type Output = $name<T>;

            fn div(self, rhs: &T) -> Self::Output {
                let rhs = *rhs;
                $name::<T>::new($(self.$component / rhs),*)
            }
        }

        forward_ref_binop!($name<T>, Div, div, T, $name<T>);
        forward_ref_binop_assign!($name<T>, Div, div, DivAssign, div_assign, T);

        impl<T, U: Into<T> + Copy> From<&$name<U>> for $name<T> {
            fn from(value: &$name<U>) -> Self {
                $name::<T>::new($(value.$component.into()),*)
            }
        }
    };
}

define_vector!(Vector2, x, y);
define_vector!(Vector3, x, y, z);
define_vector!(Vector4, x, y, z, w);

pub type Vector2i = Vector2<isize>;
pub type Vector3i = Vector3<isize>;
pub type Vector4i = Vector4<isize>;
pub type Vector2f = Vector2<f64>;
pub type Vector3f = Vector3<f64>;
pub type Vector4f = Vector4<f64>;

impl<T: VectorType> Cross<&Vector3<T>> for &Vector3<T> {
    type Output = Vector3<T>;

    fn cross(self, rhs: &Vector3<T>) -> Self::Output {
        Vector3::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }
}

forward_ref_binop!(Vector3<T>, Cross, cross, Vector3<T>, Vector3<T>);
forward_ref_binop_assign!(
    Vector3<T>,
    Cross,
    cross,
    CrossAssign,
    cross_assign,
    Vector3<T>
);

#[cfg(test)]
mod tests {
    use crate::math::vector::Cross;
    use crate::math::vector::Dot;
    use crate::math::vector::Vector2;
    use crate::math::vector::Vector3;

    #[test]
    fn test_vector2_mul() {
        let v1 = Vector2::new(2, 4);
        let v2 = Vector2::new(3, 5);
        assert_eq!(Vector2::new(6, 20), v1 * v2);
    }

    #[test]
    fn test_vector3_dot() {
        let v1 = Vector3::new(1, 0, 0);
        let v2 = Vector3::new(0, 1, 0);
        assert_eq!(0, v1.dot(v2));
    }

    #[test]
    fn test_vector3_cross() {
        let v1 = Vector3::new(1, 0, 0);
        let v2 = Vector3::new(0, 1, 0);
        assert_eq!(Vector3::new(0, 0, 1), v1.cross(v2));
    }
}
