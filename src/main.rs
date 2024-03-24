use std::{collections::HashMap, io::{Read, Write}};
use config::Config;
use http::{HttpRequest, HttpResponse, HttpStatusCode};
use nom::{error, Err};
use router::Router;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use std::env;
use std::fs::File;
use anyhow::{anyhow, Result};
use bytes::{BufMut, Bytes, BytesMut};

use crate::http::HttpMethod;

pub mod router;
pub mod matcher;
pub mod http;
pub mod config;

#[tokio::main]
async fn main() -> Result<()>{
    let config = Config::parse_config();
    let mut router = Router::new();
    router = router.add("*", handle_request);

    let listener = TcpListener::bind("127.0.0.1:4221").await?;

    loop {
        let (stream, _peer) = listener.accept().await?;
        let config = config.clone();
        let router = router.clone();

        tokio::spawn(async move {
            handle_client_v2(stream, router, config).await;
            // handle_client(stream).await;
        });
    }
}

async fn handle_client_v2(mut stream: TcpStream, router: Router, config: Config) {
    let mut buffer = [0; 1024];
    let result = stream.read(&mut buffer).await;
    match result {
        Ok(read) => {
            println!("read {} bytes", read);
        }
        Err(err) => {
            println!("error reading bytes: {}", err);
            return
        }
    }

    let received = std::str::from_utf8(&buffer).expect("valid utf8");

    let mut request = parse_http(received);

    let mut response = HttpResponse{code: HttpStatusCode::NotFound, body: Bytes::new(), headers: vec![]};
    if let Some((matched, handler)) = router.match_url(&request.path) {
        request.params = matched.params;
        response = handler(request, &config);
    } 

    let todo = stream.write(&construct_response(response)).await;
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

async fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let result = stream.read(&mut buffer).await;
    match result {
        Ok(read) => {
            println!("read {} bytes", read);
        }
        Err(e) => {
            panic!("error: {}", e);
        }
    }

    let received = std::str::from_utf8(&buffer).expect("valid utf8");

    let request = parse_http(received);

    if request.path == "/" {
        stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").await.expect("writing to stream");
        return
    }

    if request.path.starts_with("/echo/") {
        let query = request.path.strip_prefix("/echo/").expect("trimmed");
        let content_length = format!("Content-Length: {}\r\n\r\n", query.len());
        stream.write_all(["HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n", content_length.as_str(), query].concat().as_bytes()).await.expect("writing to stream");
        return
    }

    if request.path.starts_with("/files/") {
        let args: Vec<String> = env::args().collect();
        let mut directory = None;
    
        for i in 1..args.len() {
            if args[i] == "--directory" {
                // If the current argument is --directory or -d, try to get the next argument as the directory
                if let Some(dir) = args.get(i + 1) {
                    directory = Some(dir.clone());
                } else {
                    println!("Error: Missing directory argument.");
                    return;
                }
            }
        }
        let dir = directory.expect("directory arg");

        let query = request.path.strip_prefix("/files/").expect("trimmed");
        let file_path = format!("{}/{}", dir, query);

        if let http::HttpMethod::Post = request.method {
            match File::create(file_path) {
                Ok(mut file) => {
                    // Write data to the file
                    match file.write_all(request.body.as_bytes()) {
                        Ok(_) => println!("Data written to file successfully."),
                        Err(e) => panic!("Error writing to file: {}", e),
                    }

                    stream.write_all(b"HTTP/1.1 201 OK\r\nContent-Type: application/octet-stream\r\n\r\n").await.expect("writing to stream");
                    return
                }
                Err(e) => println!("Error creating file: {}", e),
            }

        } else {
            match File::open(&file_path) {
                Ok(mut file) => {
                    let mut contents = String::new();
                    // Read the contents of the file into a string
                    match file.read_to_string(&mut contents) {
                        Ok(_) => {},
                        Err(e) => panic!("Error reading file: {}", e),
                    }
                    let content_length = format!("Content-Length: {}\r\n\r\n", contents.len());
                    stream.write_all(["HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\n", content_length.as_str(), contents.as_str()].concat().as_bytes()).await.expect("writing to stream");
                }
                Err(_) => {
                    stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").await.expect("writing to stream");
                    return
                },
            }    
        }

    }
    if request.path.starts_with("/") {
        let query = request.path.strip_prefix("/").expect("trimmed");

        if let Some(response) = request.headers.get(&query.to_string().to_lowercase()) {
            let content_length = format!("Content-Length: {}\r\n\r\n", response.len());
            stream.write_all(["HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n", content_length.as_str(), response].concat().as_bytes()).await.expect("writing to stream");
        }
    }

    stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").await.expect("writing to stream");
}

fn parse_http(data: &str) -> HttpRequest {
    let mut request = HttpRequest{path: String::new(), params: HashMap::new(),method: HttpMethod::Unimplemented, headers: HashMap::new(), body: String::new()};
    let mut lines = data.lines();

    let request_line = lines.next();

    match request_line {
        Some(request_line) => {
            let mut splitted = request_line.split(' ');
            let method =  splitted.next();
            let path = splitted.next();

            match method {
                Some(method) => {
                    match method {
                        "GET" => { request.method = HttpMethod::Get }
                        "POST" =>{ request.method = HttpMethod::Post }
                        _ =>{ }
                    }
                }
                None => {}
            }

            match path {
                Some(path) => {request.path = path.into()}
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
            request.headers.insert(name.to_string().to_lowercase(), value.to_string());
        }
    }

    request.body = header_lines.last().expect("body").trim_matches(char::from(0)).to_string();

    return request
}


fn handle_request(r: HttpRequest, config: &Config) -> HttpResponse {
    HttpResponse{code: HttpStatusCode::Success, body: Bytes::new(), headers: vec![]}
}
