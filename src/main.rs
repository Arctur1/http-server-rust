use anyhow::Result;
use bytes::{BufMut, Bytes, BytesMut};
use config::Config;
use http::{HttpRequest, HttpResponse, HttpStatusCode};
use router::{http::Header, Router};
use std::fs::File;
use std::{
    collections::HashMap,
    io::{Read, Write},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;

use crate::http::HttpMethod;

pub mod config;
pub mod http;
pub mod matcher;
pub mod router;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::parse_config();
    let mut router = Router::new();
    router = router
        .add("/", ok)
        .add("/echo/*", echo)
        .add("/files/:file", file)
        .add("/:header", headers);

    let listener = TcpListener::bind("127.0.0.1:4221").await?;

    loop {
        let (stream, _peer) = listener.accept().await?;
        let config = config.clone();
        let router = router.clone();

        tokio::spawn(async move {
            handle_client(stream, router, config).await;
        });
    }
}

async fn handle_client(mut stream: TcpStream, router: Router, config: Config) {
    let mut buffer = [0; 1024];
    let result = stream.read(&mut buffer).await;
    match result {
        Ok(read) => {
            println!("read {} bytes", read);
        }
        Err(err) => {
            println!("error reading bytes: {}", err);
            return;
        }
    }

    let received = std::str::from_utf8(&buffer).expect("valid utf8");

    let mut request = parse_http(received);

    let mut response = HttpResponse {
        code: HttpStatusCode::NotFound,
        body: Bytes::new(),
        headers: vec![],
    };
    if let Some((matched, handler)) = router.match_url(&request.path) {
        request.params = matched.params;
        response = handler(request, &config);
    }

    let _todo = stream.write(&construct_response(response)).await;
}

fn construct_response(res: HttpResponse) -> Bytes {
    let mut buffer = BytesMut::new();
    buffer.put(res.code.header());
    buffer.put(&b"\r\n"[..]);

    for header in res.headers {
        buffer.put(header.name);
        buffer.put(&b": "[..]);
        buffer.put(header.value);
        buffer.put(&b"\r\n"[..]);
    }

    buffer.put(&b"\r\n"[..]);
    buffer.put(res.body);

    buffer.into()
}

fn file(request: HttpRequest, config: &Config) -> HttpResponse {
    if let http::HttpMethod::Post = request.method {
        return post_file(request, config);
    } else {
        return get_file(request, config);
    }
}

fn post_file(request: HttpRequest, config: &Config) -> HttpResponse {
    let dir = config.directory.as_ref().expect("passed dir");

    let query = request.path.strip_prefix("/files/").expect("trimmed");
    let file_path = format!("{}/{}", dir, query);

    match File::create(file_path) {
        Ok(mut file) => {
            // Write data to the file
            match file.write_all(request.body.as_bytes()) {
                Ok(_) => (),
                Err(e) => {
                    let body = format!("Error writing to file: {}", e);
                    let content_length = body.len();
                    return HttpResponse {
                        code: HttpStatusCode::ServerError,
                        body: body.into(),
                        headers: vec![Header {
                            name: "Content-Length".into(),
                            value: format!("{}", content_length).into(),
                        }],
                    };
                }
            }

            return HttpResponse {
                code: HttpStatusCode::Created,
                body: Bytes::new(),
                headers: vec![],
            };
        }

        Err(e) => {
            let body = format!("Error creating file: {}", e);
            let content_length = body.len();
            return HttpResponse {
                code: HttpStatusCode::ServerError,
                body: body.into(),
                headers: vec![Header {
                    name: "Content-Length".into(),
                    value: format!("{}", content_length).into(),
                }],
            };
        }
    }
}

fn get_file(request: HttpRequest, config: &Config) -> HttpResponse {
    let dir = config.directory.as_ref().expect("passed dir").clone();

    let query = request.path.strip_prefix("/files/").expect("trimmed");
    let file_path = format!("{}/{}", dir, query);
    match File::open(&file_path) {
        Ok(mut file) => {
            let mut contents = String::new();
            match file.read_to_string(&mut contents) {
                Ok(_) => {}
                Err(e) => {
                    let body = format!("Error reading file: {}", e);
                    let content_length = body.len();
                    return HttpResponse {
                        code: HttpStatusCode::ServerError,
                        body: body.into(),
                        headers: vec![Header {
                            name: "Content-Length".into(),
                            value: format!("{}", content_length).into(),
                        }],
                    };
                }
            }
            let content_length = contents.len();
            return HttpResponse {
                code: HttpStatusCode::Success,
                body: contents.into(),
                headers: vec![
                    Header {
                        name: "Content-Length".into(),
                        value: format!("{}", content_length).into(),
                    },
                    Header {
                        name: "Content-Type".into(),
                        value: "application/octet-stream".into(),
                    },
                ],
            };
        }
        Err(_) => {
            return HttpResponse {
                code: HttpStatusCode::NotFound,
                body: Bytes::new(),
                headers: vec![],
            }
        }
    }
}

fn echo(request: HttpRequest, _: &Config) -> HttpResponse {
    let body = request
        .path
        .strip_prefix("/echo/")
        .expect("trimmed")
        .to_string();
    let content_length = body.len();
    return HttpResponse {
        code: HttpStatusCode::Success,
        body: body.into(),
        headers: vec![
            Header {
                name: "Content-Length".into(),
                value: format!("{}", content_length).into(),
            },
            Header {
                name: "Content-Type".into(),
                value: "text/plain".into(),
            },
        ],
    };
}

fn headers(request: HttpRequest, _: &Config) -> HttpResponse {
    let query = request.path.strip_prefix("/").expect("trimmed");
    let header = request.headers.get(&query.to_string().to_lowercase());

    if let Some(header) = header {
        return HttpResponse {
            code: HttpStatusCode::Success,
            body: header.clone().into(),
            headers: vec![
                Header {
                    name: "Content-Length".into(),
                    value: format!("{}", header.len()).into(),
                },
                Header {
                    name: "Content-Type".into(),
                    value: "text/plain".into(),
                },
            ],
        };
    }
    return HttpResponse {
        code: HttpStatusCode::NotFound,
        body: Bytes::new(),
        headers: vec![],
    };
}

fn ok(_: HttpRequest, _: &Config) -> HttpResponse {
    return HttpResponse {
        code: HttpStatusCode::Success,
        body: Bytes::new(),
        headers: vec![],
    };
}

fn parse_http(data: &str) -> HttpRequest {
    let mut request = HttpRequest {
        path: String::new(),
        params: HashMap::new(),
        method: HttpMethod::Unimplemented,
        headers: HashMap::new(),
        body: String::new(),
    };
    let mut lines = data.lines();

    let request_line = lines.next();

    match request_line {
        Some(request_line) => {
            let mut splitted = request_line.split(' ');
            let method = splitted.next();
            let path = splitted.next();

            match method {
                Some(method) => match method {
                    "GET" => request.method = HttpMethod::Get,
                    "POST" => request.method = HttpMethod::Post,
                    _ => {}
                },
                None => {}
            }

            match path {
                Some(path) => request.path = path.into(),
                None => {}
            }

            println!("read {} bytes", request_line);
        }
        None => {
            return request;
        }
    }

    let header_lines: Vec<&str> = lines.collect();
    for header_line in header_lines.iter() {
        if let Some((name, value)) = header_line.split_once(": ") {
            request
                .headers
                .insert(name.to_string().to_lowercase(), value.to_string());
        }
    }

    request.body = header_lines
        .last()
        .expect("body")
        .trim_matches(char::from(0))
        .to_string();

    return request;
}
