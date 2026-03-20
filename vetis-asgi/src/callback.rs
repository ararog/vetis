use pyo3::{pyclass, pymethods, types::PyDict, Bound, PyAny, PyResult};

#[pyclass]
pub(crate) struct Receive {}

#[pymethods]
impl Receive {
    fn __call__(&mut self) -> PyResult<()> {
        // TODO: Handle the receive
        Ok(())
    }
}

#[pyclass]
pub(crate) struct Send {}

#[pymethods]
impl Send {
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
        receive: &Bound<'py, PyAny>,
        send: &Bound<'py, PyAny>,
    ) -> PyResult<()> {
        // TODO: Handle the scope
        Ok(())
    }
}
