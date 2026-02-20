use clap::Parser;
use log::error;

use serde::Deserialize;

#[cfg(feature = "smol-rt")]
use macro_rules_attribute::apply;
#[cfg(feature = "smol-rt")]
use smol_macros::main;

use std::{error::Error, fs::read_to_string, path::Path};
use vetis::{
    config::server::{virtual_host::VirtualHostConfig, ServerConfig},
    server::virtual_host::VirtualHost,
    Vetis,
};

#[derive(Deserialize)]
pub struct VetisServerConfig {
    log_level: String,
    workers: usize,
    max_blocking_threads: usize,
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

async fn run(
    server_config: ServerConfig,
    virtual_hosts_config: Vec<VirtualHostConfig>,
) -> Result<(), Box<dyn Error>> {
    let mut server = Vetis::new(server_config);

    for virtual_host in virtual_hosts_config {
        let virtual_host = VirtualHost::new(virtual_host);

        server
            .add_virtual_host(virtual_host)
            .await;
    }

    if let Err(e) = server.run().await {
        error!("Failed to start server: {}", e);
    }

    Ok(())
}

fn init_runtime() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    if let Some(config) = args.config {
        if Path::exists(Path::new(&config)) {
            let file = read_to_string(&config);
            if let Err(e) = file {
                return Err(e.into());
            }

            if let Ok(file) = file {
                let config = serde_yaml_ng::from_str::<VetisServerConfig>(&file);
                if let Err(e) = config {
                    return Err(e.into());
                }

                if let Ok(config) = config {
                    env_logger::Builder::from_env(
                        env_logger::Env::default().filter_or("RUST_LOG", config.log_level),
                    )
                    .format_module_path(false)
                    .init();

                    #[cfg(feature = "tokio-rt")]
                    {
                        let rt = tokio::runtime::Builder::new_multi_thread()
                            .enable_all()
                            .worker_threads(config.workers)
                            .max_blocking_threads(config.max_blocking_threads)
                            .build()?;
                        rt.block_on(async { run(config.server, config.virtual_hosts).await })?;
                    }

                    #[cfg(feature = "smol-rt")]
                    {
                        smol::block_on(async { run(config.server, config.virtual_hosts).await })?;
                    }
                } else {
                    eprintln!(
                        "Failed to parse config file: {}",
                        config
                            .err()
                            .unwrap()
                    );
                }
            }
        }
    }

    Ok(())
}

#[cfg(feature = "tokio-rt")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = init_runtime() {
        eprintln!("Failed to start server: {}", e);
    }
    Ok(())
}

#[cfg(feature = "smol-rt")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = init_runtime() {
        eprintln!("Failed to start server: {}", e);
    }
    Ok(())
}
