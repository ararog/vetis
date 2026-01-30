# VeTiS (Very Tiny Server)

🚀 **A blazingly fast, minimalist HTTP server built for modern Rust applications**

VeTiS is a lightweight yet powerful web server that brings simplicity and performance together. Designed with Rust's safety guarantees in mind, it delivers HTTP/1, HTTP/2, and HTTP/3 support with a clean, intuitive API that makes building web services a breeze.

## ✨ Why VeTiS?

- **🎯 Minimalist Design**: Focus on what matters - serving HTTP requests efficiently
- **🔧 Flexible Runtime**: Choose between Tokio or Smol async runtimes
- **🌐 Protocol Support**: Full HTTP/1, HTTP/2, and HTTP/3 implementation
- **🛡️ Secure by Default**: Built-in TLS support with modern cryptography
- **⚡ Zero-Cost Abstractions**: Leverage Rust's performance without overhead
- **📦 Feature-Gated**: Include only what you need for optimal binary size

## 🛠️ Quick Start

Add VeTiS to your `Cargo.toml`:

```toml
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

## 💡 Usage Example

Here's how simple it is to create a web server with VeTiS:

```rust
use bytes::Bytes;
use http_body_util::Full;
use hyper::StatusCode;
use vetis::{
    Vetis,
    config::{ListenerConfig, SecurityConfig, ServerConfig, VirtualHostConfig},
    server::virtual_host::{VirtualHost, VirtualHost, handler_fn},
};

pub const CA_CERT: &[u8] = include_bytes!("../certs/ca.der");
pub const SERVER_CERT: &[u8] = include_bytes!("../certs/server.der");
pub const SERVER_KEY: &[u8] = include_bytes!("../certs/server.key.der");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    std_logger::Config::logfmt().init();

    let https = ListenerConfig::builder()
        .port(8443)
        .protocol(vetis::config::Protocol::HTTP1)
        .interface("0.0.0.0".to_string())
        .build();

    let config = ServerConfig::builder()
        .add_listener(https)
        .build();

    let security_config = SecurityConfig::builder()
        .ca_cert_from_bytes(CA_CERT.to_vec())
        .cert_from_bytes(SERVER_CERT.to_vec())
        .key_from_bytes(SERVER_KEY.to_vec())
        .build();

    let localhost_config = VirtualHostConfig::builder()
        .hostname("localhost".to_string())
        .port(8443)
        .security(security_config)
        .build()?;

    let mut localhost_virtual_host = VirtualHost::new(localhost_config);

    let mut root_path = HandlerPath::new("/", handler_fn(|request| async move {
         let response = vetis::Response::builder()
             .status(StatusCode::OK)
             .body(Full::new(Bytes::from("Hello, World!")));
         Ok(response)
    }));
     
    localhost_virtual_host.add_path(root_path);    

    let mut server = Vetis::new(config);
    server
        .add_virtual_host(localhost_virtual_host)
        .await;

    server.run().await?;

    server
        .stop()
        .await?;

    Ok(())
}
```

## 🗺️ Roadmap

VeTiS is continuously evolving! Here's what we're working on:

### Core Features

- **🌐 Virtual Hosts** - Host multiple domains on a single server
- **🔌 WebSockets** - Real-time bidirectional communication
- **🔐 SNI Support** - Server Name Indication for TLS
- **🔄 Reverse Proxy** - Route requests to backend services
- **⚖️ Load Balancing** - Distribute traffic across multiple servers

### Content & Security

- **📁 Static File Serving** - Efficient static asset delivery
- **🎭 Dynamic Content** - Template rendering and content generation
- **🔑 Authentication** - Multiple auth methods support
- **🛡️ Authorization** - Fine-grained access control
- **📊 Logging** - Comprehensive request and error logging

## 📄 License

MIT

## 👤 Author

Rogerio Pereira Araujo <rogerio.araujo@gmail.com>
