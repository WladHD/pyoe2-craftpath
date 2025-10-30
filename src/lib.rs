#[cfg(feature = "python")]
pub mod py {
    use polars::prelude::*;
    use polars::{frame::DataFrame, series::Series};
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;
    use pyo3_polars::PyDataFrame;
    use pyo3_stub_gen::define_stub_info_gatherer;

    #[pyfunction]
    fn create_animal_df() -> PyResult<PyDataFrame> {
        let ids = Series::new("id".into(), &[1u64, 2, 3]);
        let names = Series::new("animal_name".into(), &["dog", "cat", "birb"]);

        let df = DataFrame::new(vec![ids.into(), names.into()])
            .map_err(|err| PyRuntimeError::new_err(err.to_string()))?;

        let df = pyo3_polars::PyDataFrame(df);
        Ok(df)
    }

    #[pymodule]
    fn pyoe2_craftpath(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(create_animal_df, m)?)?;
        Ok(())
    }

    define_stub_info_gatherer!(stub_info);
}
