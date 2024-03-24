use std::collections::HashMap;

use bytes::Bytes;



pub enum HttpStatusCode {
    Success,
    NotFound,
}

pub struct HttpResponse {
    pub code: HttpStatusCode,
    pub body: Bytes,
    pub headers: Vec<Header>
}

pub struct Header {
    pub name: Bytes,
    pub value: Bytes
}

impl HttpStatusCode {
    pub fn header(self) -> Bytes {
        match self {
            Self::Success => { "HTTP/1.1 200 OK".into() }
            Self::NotFound =>  { "HTTP/1.1 404 Not Found".into() }
        }
    }
}

pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub params: HashMap<String, String>,
    pub body: String,
}

pub enum HttpMethod {
    Unimplemented,
    Get,
    Post,
}
