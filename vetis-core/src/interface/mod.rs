use std::{pin::Pin, sync::Arc};

use crate::{
    errors::VetisError,
    http::{Request, Response},
};

pub trait InterfaceWorker {
    fn handle(
        &self,
        request: Arc<Request>,
        uri: Arc<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + 'static>>;
}
