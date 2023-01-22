pub mod job;
pub mod logger;

#[macro_export]
macro_rules! smart_enum {
    ($v:vis, $i:ident, $($j:ident),*) => {
        #[derive(Copy, Clone, Eq, Hash, PartialEq)]
        $v enum $i {
            $($j),*
        }

        impl $i {
            pub fn to_string(&self) -> &str {
                (*self).into()
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
