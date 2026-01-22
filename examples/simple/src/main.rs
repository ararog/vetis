use bytes::Bytes;
use clap::Parser;
use http_body_util::Full;
use hyper::Response;
use vetis::{server::config::{SecurityConfig, ServerConfig}, Vetis};

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
        default_value = "8443",
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let interface = args.interface;
    let port = args.port;

    let security = SecurityConfig::builder()
        .cert(SERVER_CERT.to_vec())
        .key(SERVER_KEY.to_vec())
        .build();

    let config = ServerConfig::builder()
        .port(port)
        .interface(interface)
        .security(security)
        .build();

    let mut server = Vetis::new(config);

    server
        .run(|_| async move { Ok(Response::new(Full::new(Bytes::from("Hello World")))) })
        .await?;

    server.stop().await?;

    Ok(())
}
