use log::error;
use std::{error::Error, fs::read_to_string, path::Path};
use vetis::{Vetis, CONFIG};

async fn run() -> Result<(), Box<dyn Error>> {
    if Path::exists(Path::new(CONFIG)) {
        let file = read_to_string(CONFIG);
        if let Ok(file) = file {
            let config = toml::from_str(&file);
            if let Ok(config) = config {
                let mut server = Vetis::new(config);
                if let Err(e) = server.run().await {
                    error!("Failed to start server: {}", e);
                }
            } else {
                error!("Failed to parse config file");
            }
        }
    }
    Ok(())
}

#[cfg(feature = "tokio-rt")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run().await
}

#[cfg(feature = "smol-rt")]
#[apply::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run().await
}
