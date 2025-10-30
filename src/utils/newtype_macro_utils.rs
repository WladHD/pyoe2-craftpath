#[macro_export]
macro_rules! explicit_type {
    // Special case for floats (f32, f64) cauz no Eq, Ord, Hash
    ($name:ident, f32) => {
        #[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
        pub struct $name(f32);

        impl From<f32> for $name {
            fn from(value: f32) -> Self {
                $name(value)
            }
        }

        impl From<$name> for f32 {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl $name {
            pub fn get_raw_value(&self) -> &f32 {
                &self.0
            }
        }
    };
    ($name:ident, f64) => {
        #[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
        pub struct $name(f64);

        impl From<f64> for $name {
            fn from(value: f64) -> Self {
                $name(value)
            }
        }

        impl From<$name> for f64 {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl $name {
            pub fn get_raw_value(&self) -> &f64 {
                &self.0
            }
        }
    };

    // normal case
    ($name:ident, $inner:ty) => {
        #[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
        pub struct $name($inner);

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                $name(value)
            }
        }

        impl From<$name> for $inner {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl $name {
            pub fn get_raw_value(&self) -> &$inner {
                &self.0
            }
        }
    };
}