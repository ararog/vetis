use bytes::Bytes;
use clap::Parser;
use http_body_util::Full;
use hyper::{Response, body::Incoming};
use vetis::{
    Vetis,
    server::{
        config::{SecurityConfig, ServerConfig, VirtualHostConfig},
        errors::VetisError,
        virtual_host::{DefaultVirtualHost, VirtualHost},
    },
};

pub const SERVER_CERT: &[u8] = include_bytes!("../certs/server.der");
pub const SERVER_KEY: &[u8] = include_bytes!("../certs/server.key.der");

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(
        short = 'p',
        long,
        required = false,
        num_args = 0..=1,
        require_equals = true,
        default_value = "8080",
        help = "Set bearer auth token on Authorization header."
    )]
    port: u16,

    #[arg(
        short = 'i',
        long,
        required = false,
        num_args = 0..=1,
        require_equals = true,
        default_value = "0.0.0.0",
        help = "Set bearer auth token on Authorization header."
    )]
    interface: String,

    #[arg(
        short = 'c',
        long,
        required = false,
        num_args = 0..=1,
        require_equals = true,
        default_value = "../certs/server.der",
        help = "Set server certificate file (DER encoded)."
    )]
    cert: String,

    #[arg(
        short = 'k',
        long,
        required = false,
        num_args = 0..=1,
        require_equals = true,
        default_value = "../certs/server.key.der",
        help = "Set server certificate file (DER encoded)."
    )]
    key: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    std_logger::Config::logfmt().init();

    let args = Args::parse();

    let interface = args.interface;
    let port = args.port;

    let config = ServerConfig::builder()
        .port(port)
        .interface(interface)
        .build();


    let localhost_config = VirtualHostConfig::builder()
        .hostname("localhost:8080".to_string())
        .build();

    let server_config = VirtualHostConfig::builder()
        .hostname("server:8080".to_string())
        .build();

    let localhost_virtual_host = DefaultVirtualHost::new(localhost_config, Box::new(|request| Box::pin(async move {
        Ok(Response::new(Full::new(Bytes::from("Hello from localhost"))))
    })));

    let server_virtual_host = DefaultVirtualHost::new(server_config, Box::new(|request| Box::pin(async move {
        Ok(Response::new(Full::new(Bytes::from("Hello from server"))))
    })));

    let mut server = Vetis::new(config);
    server.add_virtual_host(Box::new(localhost_virtual_host)).await;
    server.add_virtual_host(Box::new(server_virtual_host)).await;

    server.run().await?;

    server
        .stop()
        .await?;

    Ok(())
}
