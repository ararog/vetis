use vetis::{Vetis, server::config::ServerConfig};
use http_body_util::{Full};
use bytes::Bytes;
use hyper::Response;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = 8080;
    let interface = "::".to_string();

    let config = ServerConfig::builder()
        .port(port)
        .interface(interface)
        .build();


    let mut server = Vetis::new(config);

    server.run(|_| async move {
        Ok(Response::new(Full::new(Bytes::from("Hello World"))))
    }).await?;

    Ok(())
}
