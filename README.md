# VeTiS (Very Tiny Server)

Very tiny server.

## Description

**Vetis** is a small webserver.

## Install

```rust
vetis = { version = "0.1.0", features = ["tokio-rt", "http2", "tokio-rust-tls"] }
```

## Runtimes

- [tokio](https://github.com/tokio-rs/tokio)
- [smol](https://github.com/smol-rs/smol)

## Crate features

- tokio-rt (default)
- smol-rt
- http1
- http2 (default)
- http3
- tokio-rust-tls (default)

## Examples

```rust
use bytes::Bytes;
use clap::Parser;
use http_body_util::Full;
use hyper::Response;
use vetis::{server::config::ServerConfig, Vetis};

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let interface = args.interface;
    let port = args.port;

    let config = ServerConfig::builder()
        .port(port)
        .interface(interface)
        .build();

    let mut server = Vetis::new(config);

    server
        .run(|_| async move { Ok(Response::new(Full::new(Bytes::from("Hello World")))) })
        .await?;

    server.stop().await?;

    Ok(())
}
```

## License

MIT

## Author

Rogerio Pereira Araujo <rogerio.araujo@gmail.com>

