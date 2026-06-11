#[macro_export]
macro_rules! explicit_type {
    // Special case for floats (f32, f64) cauz no Eq, Ord, Hash
    ($name:ident, f32) => {
        #[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
        #[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
        #[cfg_attr(feature = "python", pyo3::pyclass)]
        #[cfg_attr(feature = "python", pyo3(eq, weakref, from_py_object, str))]
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

        #[cfg(feature = "python")]
        #[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pymethods)]
        #[cfg_attr(feature = "python", pyo3::pymethods)]
        impl $name {
            #[new]
            pub fn new(value: f32) -> Self {
                Self::from(value)
            }

            pub fn get_raw_value(&self) -> &f32 {
                &self.0
            }
        }

        #[cfg(not(feature = "python"))]
        impl $name {
            pub fn new(value: f32) -> Self {
                Self::from(value)
            }

            pub fn get_raw_value(&self) -> &f32 {
                &self.0
            }
        }

        #[cfg(feature = "python")]
        crate::derive_DebugDisplay!($name);
    };

    ($name:ident, f64) => {
        #[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
        #[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
        #[cfg_attr(feature = "python", pyo3::pyclass)]
        #[cfg_attr(feature = "python", pyo3(eq, weakref, from_py_object, str))]
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

        #[cfg(feature = "python")]
        #[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pymethods)]
        #[cfg_attr(feature = "python", pyo3::pymethods)]
        impl $name {
            #[new]
            pub fn new(value: f64) -> Self {
                Self::from(value)
            }

            pub fn get_raw_value(&self) -> &f64 {
                &self.0
            }
        }

        #[cfg(not(feature = "python"))]
        impl $name {
            pub fn new(value: f64) -> Self {
                Self::from(value)
            }

            pub fn get_raw_value(&self) -> &f64 {
                &self.0
            }
        }

        #[cfg(feature = "python")]
        crate::derive_DebugDisplay!($name);
    };

    // normal case
    ($name:ident, $inner:ty) => {
        #[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
        #[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
        #[cfg_attr(feature = "python", pyo3::pyclass)]
        #[cfg_attr(feature = "python", pyo3(eq, weakref, from_py_object, frozen, hash, str))]
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

        #[cfg(feature = "python")]
        #[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pymethods)]
        #[cfg_attr(feature = "python", pyo3::pymethods)]
        impl $name {
            #[new]
            pub fn new(value: $inner) -> Self {
                Self::from(value)
            }

            #[getter]
            pub fn get_raw_value(&self) -> &$inner {
                &self.0
            }
        }

        #[cfg(not(feature = "python"))]
        impl $name {
            pub fn new(value: $inner) -> Self {
                Self::from(value)
            }

            pub fn get_raw_value(&self) -> &$inner {
                &self.0
            }
        }

        #[cfg(feature = "python")]
        crate::derive_DebugDisplay!($name);
    };
}
