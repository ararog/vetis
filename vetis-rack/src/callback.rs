use magnus::wrap;
use std::{collections::HashMap, result::Result};

#[wrap(class = "Application")]
pub(crate) struct Application {}

impl Application {
    fn call(&mut self, env: HashMap<String, String>) -> Result<(), String> {
        // TODO: Handle the scope
        Ok(())
    }
}
