use std::{collections::HashMap, io::{Read, Write}, net::{TcpListener, TcpStream}};


fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let result = stream.read(&mut buffer);
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
        stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").expect("writing to stream");
        return
    }

    if request.path.starts_with("/echo/") {
        let query = request.path.strip_prefix("/echo/").expect("trimmed");
        let content_length = format!("Content-Length: {}\r\n\r\n", query.len());
        stream.write_all(["HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n", content_length.as_str(), query].concat().as_bytes()).expect("writing to stream");
        return
    }

    if request.path.starts_with("/") {
        let query = request.path.strip_prefix("/").expect("trimmed");

        if let Some(response) = request.headers.get(&query.to_string().to_lowercase()) {
            let content_length = format!("Content-Length: {}\r\n\r\n", response.len());
            stream.write_all(["HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n", content_length.as_str(), response].concat().as_bytes()).expect("writing to stream");
        }


    }
    stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").expect("writing to stream");
}

fn parse_http(data: &str) -> HttpRequest {
    let mut request = HttpRequest{path: String::new(), method: HttpMethod::Unimplemented, headers: HashMap::new()};
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
    for header_line in header_lines {
        if let Some((name, value)) = header_line.split_once(": ") {
            request.headers.insert(name.to_string().to_lowercase(), value.to_string());
        }
    }


    return request
}

struct HttpRequest {
    method: HttpMethod,
    path: String,
    headers: HashMap<String, String>,
}

enum HttpMethod {
    Unimplemented,
    Get,
}
