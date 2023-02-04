pub mod job;
pub mod logger;

#[macro_export]
macro_rules! smart_enum {
    ($v:vis, $i:ident, $($j:ident),*) => {
        #[derive(Copy, Clone, Eq, Hash, PartialEq)]
        $v enum $i {
            $($j),*
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
