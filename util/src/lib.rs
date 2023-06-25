pub mod job;
pub mod logger;
pub mod math;
pub mod runtime_id;

#[macro_export]
macro_rules! smart_enum {
    ($v:vis, $i:ident, $($j:ident),*) => {
        #[derive(Copy, Clone, Eq, Hash, PartialEq)]
        $v enum $i {
            $($j),*
        }

        impl $i {
            pub fn values() -> Vec<Self> {
                vec![$($i::$j),*]
            }
        }

        impl std::fmt::Display for $i {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", <&str>::from(*self))
            }
        }

        impl From<$i> for &str {
            fn from(value: $i) -> Self {
                match value {
                    $($i::$j => stringify!($j)),*
                }
            }
        }
    };
}

#[macro_export]
macro_rules! internal_mut_struct {
    ($struct_name:ident, $impl_struct_name:ty $(, $field_name:ident: $field_type: ty)*) => {
        pub struct $struct_name {
            inner: Mutex<$impl_struct_name>,
            $($field_name: $field_type),*
        }

        impl $struct_name {
            fn lock_inner(&self) -> MutexGuard<$impl_struct_name> {
                self.inner.lock().unwrap()
            }
        }
    };
}

#[macro_export]
macro_rules! forward_ref_binop {
    (
        impl $([$($generic: tt)+] )?
        $op: ident,
        $fn: ident
        for
        $lhs: ty,
        $rhs: ty
        $(where $($bound:tt)+)?
    ) => {
        impl$(<$($generic)*>)? $op<&$rhs> for $lhs
        $(where $($bound)*)? {

            type Output = <&'static $lhs as $op<&'static $rhs>>::Output;

            fn $fn(self, rhs: &$rhs) -> Self::Output {
                <&$lhs>::$fn(&self, rhs)
            }
        }

        impl$(<$($generic)*>)? $op<$rhs> for &$lhs
        $(where $($bound)*)? {
            type Output = <&'static $lhs as $op<&'static $rhs>>::Output;

            fn $fn(self, rhs: $rhs) -> Self::Output {
                <&$lhs>::$fn(self, &rhs)
            }
        }

        impl$(<$($generic)*>)? $op<$rhs> for $lhs
        $(where $($bound)*)? {
            type Output = <&'static $lhs as $op<&'static $rhs>>::Output;

            fn $fn(self, rhs: $rhs) -> Self::Output {
                <&$lhs>::$fn(&self, &rhs)
            }
        }
    };
}

#[macro_export]
macro_rules! forward_ref_binop_assign {
    (
        impl $([$($generic: tt)+] )?
        $op: ident,
        $fn: ident,
        $assign_op: ident,
        $assign_fn: ident
        for
        $lhs: ty,
        $rhs: ty
        $(where $($bound:tt)+)?
    ) => {
        impl$(<$($generic)*>)? $assign_op<$rhs> for $lhs
        $(where $($bound)*)? {
            fn $assign_fn(&mut self, rhs: $rhs) {
                *self = <&$lhs>::$fn(&*self, rhs);
            }
        }

        impl$(<$($generic)*>)? $assign_op<&$rhs> for $lhs
        $(where $($bound)*)? {
            fn $assign_fn(&mut self, rhs: &$rhs) {
                *self = <&$lhs>::$fn(&*self, rhs);
            }
        }
    };
}

#[macro_export]
macro_rules! forward_ref_unop {
    (
        impl $([$($generic: tt)+] )?
        $op: ident,
        $fn: ident
        for
        $type: ty
        $(where $($bound:tt)+)?
    ) => {
        impl$(<$($generic)*>)? $op for $type
        $(where $($bound)*)? {
            type Output = <&'static $type as $op>::Output;

            fn $fn(self) -> Self::Output {
                <&$type>::$fn(&self)
            }
        }
    };
}
