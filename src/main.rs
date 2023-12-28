use std::{ net::SocketAddr, collections::HashMap };
use http_libs::{
    http::HttpMethod,
    server::{ ServerBuilder, FutureResponse },
    request::Request,
    response::Response,
};
use serde::{ Deserialize, Serialize };
use serde_json;

#[derive(Deserialize, Serialize)]
struct User {
    username: String,
    password: String,
}

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

fn api_login(_req: Request) -> FutureResponse<'static> {
    let response = Response {
        version: String::from("HTTP/1.1"),
        status_code: 200,
        status_message: String::from("OK"),
        headers: {
            let mut headers = HashMap::new();
            headers.insert(String::from("Content-Type"), String::from("application/json"));
            headers
        },
        body: {
            let user = User {
                username: String::from("Isaac"),
                password: String::from("123456"),
            };
            Some(serde_json::to_string(&user).unwrap())
        },
    };
    Box::pin(async move { Ok(response) })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let _server = ServerBuilder::new()
        .bind(addr)
        .route(HttpMethod::GET, "/", hello_handler)
        .route(HttpMethod::GET, "/api", api_login)
        .build()?
        .run().await?;

    println!("Hello world!");
    Ok(())
}
