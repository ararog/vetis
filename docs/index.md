---
layout: default
title: VeTiS - Very Tiny Server
nav_order: 1
description: "🚀 A blazingly fast, minimalist HTTP server built for modern Rust applications"
permalink: /
---
<div align="center">
<h1><b>VeTiS</b></h1>
</div>

[![crates.io](https://img.shields.io/crates/v/vetis?style=flat-square)](https://crates.io/crates/vetis)
[![Build Status](https://github.com/ararog/vetis/actions/workflows/rust.yml/badge.svg?event=push)](https://github.com/ararog/vetis/actions/workflows/rust.yml)
[![Documentation](https://docs.rs/vetis/badge.svg)](https://docs.rs/vetis/latest/vetis)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**VeTiS** is a lightweight yet powerful web server that brings simplicity and performance together. Designed with Rust's safety guarantees in mind, it delivers HTTP/1, HTTP/2, and HTTP/3 support with a clean, intuitive API that makes building web services a breeze.

Built on top of [hyper](https://github.com/hyperium/hyper).

## Features

- **🎯 Minimalist Design**: Focus on what matters - serving HTTP requests efficiently
- **🔧 Flexible Runtime**: Choose between Tokio or Smol async runtimes
- **🌐 Protocol Support**: Full HTTP/1, HTTP/2, and HTTP/3 implementation
- **🛡️ Secure by Default**: Built-in TLS support with modern cryptography
- **⚡ Zero-Cost Abstractions**: Leverage Rust's performance without overhead
- **📦 Feature-Gated**: Include only what you need for optimal binary size

## 🛠️ Quick Start

Add VeTiS to your `Cargo.toml`:

```rust
vetis = { version = "0.1.0", features = ["tokio-rt", "http2", "tokio-rust-tls"] }
```

Basic usage:

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

## Crates

| Crate | Description | Documentation |
|-------|-------------|---------------|
| [vetis](./vetis) | Core HTTP server library | [![docs.rs](https://img.shields.io/docsrs/vetis/latest)](https://docs.rs/vetis) |
| [vetis-macros](./vetis-macros) | Macros for Vetis | [![docs.rs](https://img.shields.io/docsrs/vetis-macros/latest)](https://docs.rs/vetis-macros) |

## Examples

Check out the [examples](./examples.md) for complete examples of how to use Vetis in your projects.

## Create project from template

You can create a new project from the template using `cargo generate`:

`cargo generate ararog/vetis-templates`

## Documentation

- [API Reference](https://docs.rs/vetis)
- [Contributing Guide](./CONTRIBUTING.md)

## License

This project is licensed under the [MIT License](./LICENSE.md).

## Author

Rogerio Pereira Araujo <rogerio.araujo@gmail.com>
