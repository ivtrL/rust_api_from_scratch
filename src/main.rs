use std::{ net::SocketAddr, collections::HashMap };
use http_libs::{
    http::HttpMethod,
    server::{ ServerBuilder, FutureResponse },
    request::Request,
    response::Response,
};

fn hello_handler(_req: Request) -> FutureResponse<'static> {
    let html = "<html><body><h1>Hello, world!</h1></body></html>";
    let response = Response {
        version: String::from("HTTP/1.1"),
        status_code: 200,
        status_message: String::from("OK"),
        headers: {
            let mut headers = HashMap::new();
            headers.insert(String::from("Content-Type"), String::from("text/html"));
            headers
        },
        body: Some(html.to_string()),
    };
    Box::pin(async move { Ok(response) })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let _server = ServerBuilder::new()
        .bind(addr)
        .route(HttpMethod::GET, "/", hello_handler)
        .build()?
        .run().await?;

    println!("Hello world!");
    Ok(())
}
