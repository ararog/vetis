---
layout: default
title: Vetis Macros - Procedural Macros
nav_order: 3
---

# Vetis Macros

Collection of procedural macros for Vetis HTTP server.

Available macros includes:

- `http!`
- `https!`

## Installation

```toml
[dependencies]
vetis-macros = "0.1.0"
```

## Usage

### http

```rust
let handler = handler_fn(|req| async move {
    Ok(vetis::Response::builder().body(http_body_util::Full::from("Hello, World!")))
});

let mut server = http!(
    hostname => "localhost",
    port => 8080,
    interface => "0.0.0.0",
    handler => handler
)
.await?;

server
    .start()
    .await?;

/// do something

server
    .stop()
    .await?;
```

### https

```rust
let handler = handler_fn(|req| async move {
    Ok(vetis::Response::builder().body(http_body_util::Full::from("Hello, World!")))
});

let mut server = https!(
    hostname => "localhost",
    port => 8080,
    interface => "0.0.0.0",
    cert => "./certs/server.crt",
    key => "./certs/server.key",
    handler => handler
)
.await?;

server
    .start()
    .await?;

/// do something

server
    .stop()
    .await?;  
```

## API Reference

For detailed API documentation, see the [docs.rs page](https://docs.rs/vetis-macros).
