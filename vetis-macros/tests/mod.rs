use deboa::request::get;
use vetis::server::virtual_host::handler_fn;
use vetis_macros::http;

#[cfg(feature = "smol-rt")]
use macro_rules_attribute::apply;
#[cfg(feature = "smol-rt")]
use smol_macros::test;

async fn do_test_http() -> Result<(), Box<dyn std::error::Error>> {
    let handler = handler_fn(|_req| async move {
        Ok(vetis::server::http::Response::builder().text("Hello, World!"))
    });

    let mut server = http!(
        hostname => "localhost",
        root_directory => "src",
        port => 8080,
        interface => "0.0.0.0",
        handler => handler
    )
    .await?;

    server
        .start()
        .await?;

    let client = deboa::Client::default();

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

#[cfg(feature = "tokio-rt")]
#[tokio::test]
async fn test_http() -> Result<(), Box<dyn std::error::Error>> {
    do_test_http().await
}

#[cfg(feature = "smol-rt")]
#[apply(test!)]
async fn test_http_smol() -> Result<(), Box<dyn std::error::Error>> {
    do_test_http().await
}
