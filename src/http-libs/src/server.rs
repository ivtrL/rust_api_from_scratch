use crate::request::Request;
use crate::response::Response;
use crate::http::*;

use httparse;
use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use tokio::io::{ AsyncWriteExt, AsyncReadExt };
use tokio::net::TcpListener;

pub type FutureResponse<'a> = Pin<
    Box<dyn Future<Output = Result<Response, HttpError>> + Send + 'a>
>;

pub type Handler = fn(Request) -> FutureResponse<'static>;

#[derive(Eq, Hash, PartialEq, Clone)]
struct Route {
    method: HttpMethod,
    path: String,
}

#[derive(Clone)]
pub struct Server {
    address: SocketAddr,
    routes: HashMap<Route, Handler>,
}

pub struct ServerBuilder {
    address: Option<SocketAddr>,
    routes: Option<HashMap<Route, Handler>>,
}

impl ServerBuilder {
    pub fn new() -> Self {
        Self {
            address: None,
            routes: Some(HashMap::new()),
        }
    }

    pub fn bind(mut self, socket: SocketAddr) -> Self {
        self.address = Some(socket);
        self
    }

    pub fn route(mut self, method: HttpMethod, path: &str, handler: Handler) -> Self {
        if let Some(routes) = &mut self.routes {
            routes.insert(Route { method, path: path.to_string() }, handler);
        } else {
            let mut map = HashMap::new();
            map.insert(Route { method, path: String::from(path) }, handler);
            self.routes = Some(map);
        }
        self
    }

    pub fn build(self) -> Result<Server, String> {
        let address = self.address.ok_or("Missing address")?;
        let routes = self.routes.ok_or("Missing routes")?;
        Ok(Server { address, routes })
    }
}

fn parse_request(buffer: &[u8]) -> Result<Request, Box<dyn std::error::Error>> {
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);
    let res = match req.parse(&buffer)? {
        httparse::Status::Complete(amt) => amt,
        httparse::Status::Partial => {
            return Err("Request is incomplete".into());
        }
    };

    let method = match req.method.ok_or("Missing method")? {
        "GET" => HttpMethod::GET,
        "POST" => HttpMethod::POST,
        "PUT" => HttpMethod::PUT,
        "DELETE" => HttpMethod::DELETE,
        _ => HttpMethod::OTHER(req.method.unwrap().to_string()),
    };

    let uri = req.path.ok_or("Missing uri")?.to_string();
    let version = req.version.ok_or("Missing version")?.to_string();

    let mut headers_map = HashMap::new();
    for header in req.headers.iter() {
        let name = header.name.to_string();
        let value = std::str::from_utf8(header.value)?.to_string();
        headers_map.insert(name, value);
    }

    let body = if res < buffer.len() {
        Some(String::from_utf8(buffer[res..].to_vec())?)
    } else {
        None
    };

    Ok(Request {
        method,
        uri,
        version,
        headers: headers_map,
        body,
    })
}

async fn handle_route<'a>(
    request: Request,
    routes: &'a HashMap<Route, Handler>
) -> FutureResponse<'a> {
    if
        let Some(handler) = routes.get(
            &(Route { method: request.method.clone(), path: request.uri.clone() })
        )
    {
        handler(request)
    } else {
        Box::pin(async move {
            Err(
                HttpError::InternalServerError(
                    HttpStatusCode::InternalServerError,
                    "Internal Server Error"
                )
            )
        })
    }
}

impl Server {
    pub async fn run(&self) -> std::io::Result<()> {
        let address = self.address;
        let listener = TcpListener::bind(address).await?;
        println!("Listening on {}", address.to_string());

        loop {
            let (mut stream, _) = listener.accept().await?;
            let routes = self.routes.clone();

            tokio::spawn(async move {
                let mut buffer = [0; 1024];
                let _ = stream.read(&mut buffer).await.unwrap();
                let request = parse_request(&buffer).unwrap();
                let future_response = handle_route(request, &routes).await;

                match future_response.await {
                    Ok(response) => {
                        let response_string = format!(
                            "HTTP/1.1 {} {}\r\n{}\r\n\r\n{}",
                            response.status_code,
                            response.status_message,
                            response.headers
                                .iter()
                                .map(|(k, v)| format!("{}: {}", k, v))
                                .collect::<Vec<_>>()
                                .join("\r\n"),
                            response.body.unwrap_or_default()
                        );

                        stream.write(response_string.as_bytes()).await.unwrap();
                        stream.flush().await.unwrap();
                    }

                    Err(e) => {
                        let response_string = format!("HTTP/1.1 {}", e.to_string());

                        stream.write(response_string.as_bytes()).await.unwrap();
                        stream.flush().await.unwrap();
                    }
                }
            });
        }
    }
}
