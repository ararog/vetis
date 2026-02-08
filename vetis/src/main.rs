use clap::Parser;
use log::error;
#[cfg(feature = "smol-rt")]
use macro_rules_attribute::apply;
use serde::Deserialize;
#[cfg(feature = "smol-rt")]
use smol_macros::main;
use std::{error::Error, fs::read_to_string, path::Path};
use vetis::{
    config::{ListenerConfig, ServerConfig, StaticPathConfig, VirtualHostConfig},
    server::virtual_host::VirtualHost,
    Vetis,
};

#[derive(Deserialize)]
pub struct VetisServerConfig {
    server: ServerConfig,
    virtual_hosts: Vec<VirtualHostConfig>,
}

#[derive(Parser)]
#[command(
    name = "vetis",
    about = "vetis - a very tiny server",
    long_about = r#"
vetis - a very tiny server

Usage:
    vetis [OPTIONS]

Options:
    -h, --help       Print help information
    -V, --version    Print version information
    -c, --config     <CONFIG>
                     Config file to use
"#
)]
struct Args {
    #[arg(short, long, required = false, help = "Config file to use.")]
    config: Option<String>,
}

async fn run() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().filter_or("RUST_LOG", "info")).init();

    let args = Args::parse();
    if let Some(config) = args.config {
        if Path::exists(Path::new(&config)) {
            let file = read_to_string(&config);
            if let Ok(file) = file {
                let config = toml::from_str::<VetisServerConfig>(&file);
                if let Ok(config) = config {
                    let mut server = Vetis::new(config.server);

                    for virtual_host in config.virtual_hosts {
                        let mut virtual_host = VirtualHost::new(virtual_host);

                        server
                            .add_virtual_host(virtual_host)
                            .await;
                    }

                    if let Err(e) = server.run().await {
                        error!("Failed to start server: {}", e);
                    }
                } else {
                    error!("Failed to parse config file");
                }
            }
        }
    } else {
        let listener = ListenerConfig::builder()
            .port(8080)
            .build();

        let server_config = ServerConfig::builder()
            .add_listener(listener)
            .build();

        let mut server = Vetis::new(server_config);

        let static_path_config = StaticPathConfig::builder()
            .uri("/static")
            .extensions("\\.(jpg|png|gif|html|css|js)$")
            .directory(".")
            .index_files(vec!["index.html".to_string()])
            .build()?;

        let virtual_host_config = VirtualHostConfig::builder()
            .hostname("localhost")
            .port(8080)
            .static_paths(vec![static_path_config])
            .build()?;

        let mut virtual_host = VirtualHost::new(virtual_host_config);

        server
            .add_virtual_host(virtual_host)
            .await;

        if let Err(e) = server.run().await {
            error!("Failed to start server: {}", e);
        }
    }
    Ok(())
}

#[cfg(feature = "tokio-rt")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = run().await {
        error!("Failed to start server: {}", e);
    }
    Ok(())
}

#[cfg(feature = "smol-rt")]
#[apply(main!)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = run().await {
        error!("Failed to start server: {}", e);
    }
    Ok(())
}
