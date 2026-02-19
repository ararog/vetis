use pyo3::{
    pyclass, pymethods,
    types::{PyBytes, PyBytesMethods},
    Bound, PyResult,
};

use crossfire::oneshot;
use std::ffi::CString;

pub(crate) type WsgiMessageSender = oneshot::TxOneshot<(CString, Vec<(CString, CString)>)>;

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
    fn __call__(&mut self, status: CString, headers: Vec<(CString, CString)>) -> PyResult<()> {
        if let Some(sender) = self.sender.take() {
            sender.send((status, headers));
        }
        Ok(())
    }
}
