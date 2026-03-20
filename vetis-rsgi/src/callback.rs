use pyo3::{pyclass, pymethods, types::PyDict, Bound, PyAny, PyResult};

#[pyclass]
pub(crate) struct Protocol {}

#[pymethods]
impl Protocol {
    fn __call__(&mut self) -> PyResult<()> {
        // TODO: Handle the receive
        Ok(())
    }
}

#[pyclass]
pub(crate) struct Application {}

impl Application {
    pub fn new() -> Self {
        Self {}
    }
}

#[pymethods]
impl Application {
    fn __call__<'py>(
        &mut self,
        scope: &Bound<'py, PyDict>,
        protocol: &Bound<'py, PyAny>,
    ) -> PyResult<()> {
        // TODO: Handle the scope
        Ok(())
    }
}
