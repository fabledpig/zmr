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
