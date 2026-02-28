use pyo3::{
    pyclass, pymethods,
    types::{PyBytes, PyBytesMethods},
    Bound, PyResult,
};

use crossfire::oneshot;

pub(crate) type WsgiMessageSender = oneshot::TxOneshot<(String, Vec<(String, String)>)>;

#[pyclass]
pub(crate) struct Write {
    data: Vec<u8>,
}

#[pymethods]
impl Write {
    fn __call__(&mut self, data: Bound<'_, PyBytes>) -> PyResult<()> {
        self.data
            .extend_from_slice(data.as_bytes());
        Ok(())
    }
}

#[pyclass]
pub(crate) struct StartResponse {
    sender: Option<WsgiMessageSender>,
}

impl StartResponse {
    pub fn new(sender: Option<WsgiMessageSender>) -> StartResponse {
        StartResponse { sender }
    }
}

#[pymethods]
impl StartResponse {
    fn __call__(&mut self, status: String, headers: Vec<(String, String)>) -> PyResult<()> {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send((status, headers));
        }
        Ok(())
    }
}
