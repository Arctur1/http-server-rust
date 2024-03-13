use std::{io::{Read, Write}, net::{TcpListener, TcpStream}};


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

    }
    stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").expect("writing to stream");
}

fn parse_http(data: &str) -> HttpRequest {
    let mut request = HttpRequest{path: String::new(), method: HttpMethod::Unimplemented};
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
    return request
}

struct HttpRequest {
    method: HttpMethod,
    path: String,
}

enum HttpMethod {
    Unimplemented,
    Get,
}
