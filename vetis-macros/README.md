# VeTiS Macros (Quickly build VeTiS servers)

Macros for VeTiS

## 🛠️ Quick Start

Add VeTiS Macros to your `Cargo.toml`:

```toml
vetis-macros = { version = "0.1.0", features = ["tokio-rt", "http2", "tokio-rust-tls"] }
```

## 💡 Usage Example

```rust
use deboa::request::get;
use vetis::server::virtual_host::handler_fn;
use vetis_macros::http;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handler = handler_fn(|req| async move {
        Ok(vetis::Response::builder().body(http_body_util::Full::from("Hello, World!")))
    });

    let mut server =
        http!(hostname => "localhost", port => 8080, interface => "0.0.0.0", handler => handler)
            .await?;

    // Start the server, make requests and stop the server, do not call run()!!!
    server
        .start()
        .await?;

    let client = deboa::Deboa::new();

    let response = get("http://localhost:8080")?
        .send_with(client)
        .await?;

    assert_eq!(response.status(), 200);
    assert_eq!(
        response
            .text()
            .await?,
        "Hello, World!"
    );

    server
        .stop()
        .await?;

    Ok(())
}
```

## 📄 License

MIT

## 👤 Author

Rogerio Pereira Araujo <rogerio.araujo@gmail.com>
